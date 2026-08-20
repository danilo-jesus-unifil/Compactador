use crate::error::{CoreError, CoreResult};
use crate::filesystem::{is_link_or_reparse_point, validate_inputs};
use crate::models::{CompressionLevel, InputEntry, InputKind};
use crate::security::safe_relative_path;
use crc32fast::Hasher;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::read::ZipFile;
use zip::write::{FileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

pub const DEFAULT_BUFFER_SIZE: usize = 128 * 1024;
pub const MAX_ENTRIES: usize = 1_000_000;
pub const MAX_EXPANDED_BYTES: u64 = 1_u64 << 50;
pub const MAX_COMPRESSION_RATIO: u64 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: PathBuf,
    pub is_directory: bool,
    pub original_size: u64,
    pub compressed_size: u64,
    pub checksum: u32,
    pub data_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveSummary {
    pub entries: Vec<ArchiveEntry>,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerCompression {
    Deflate,
    Store,
}

pub struct ContainerProgressCallbacks<'a> {
    pub on_progress: &'a dyn Fn(u64),
    pub on_validation_start: &'a dyn Fn(),
    pub on_finalizing_start: &'a dyn Fn(),
}

pub fn compress_file(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    level: CompressionLevel,
) -> CoreResult<ArchiveSummary> {
    compress_inputs([input.as_ref().to_path_buf()], output, level)
}

pub fn compress_inputs(
    paths: impl AsRef<[PathBuf]>,
    output: impl AsRef<Path>,
    level: CompressionLevel,
) -> CoreResult<ArchiveSummary> {
    compress_inputs_with_cancel(paths, output, level, &|| false)
}

pub fn compress_inputs_with_cancel(
    paths: impl AsRef<[PathBuf]>,
    output: impl AsRef<Path>,
    level: CompressionLevel,
    is_cancelled: &dyn Fn() -> bool,
) -> CoreResult<ArchiveSummary> {
    compress_inputs_with_strategy(
        paths,
        output,
        level,
        ContainerCompression::Deflate,
        is_cancelled,
        &|_| {},
    )
}

pub fn compress_inputs_with_strategy(
    paths: impl AsRef<[PathBuf]>,
    output: impl AsRef<Path>,
    level: CompressionLevel,
    compression: ContainerCompression,
    is_cancelled: &dyn Fn() -> bool,
    on_progress: &dyn Fn(u64),
) -> CoreResult<ArchiveSummary> {
    compress_inputs_with_strategy_and_validation(
        paths,
        output,
        level,
        compression,
        is_cancelled,
        ContainerProgressCallbacks {
            on_progress,
            on_validation_start: &|| {},
            on_finalizing_start: &|| {},
        },
    )
}

pub fn compress_inputs_with_strategy_and_validation(
    paths: impl AsRef<[PathBuf]>,
    output: impl AsRef<Path>,
    level: CompressionLevel,
    compression: ContainerCompression,
    is_cancelled: &dyn Fn() -> bool,
    callbacks: ContainerProgressCallbacks<'_>,
) -> CoreResult<ArchiveSummary> {
    let inputs = validate_inputs(paths.as_ref())?;
    let output = output.as_ref();
    ensure_output_does_not_overlap_inputs(&inputs, output)?;
    if output.exists() {
        return Err(CoreError::InvalidInput(format!(
            "o arquivo de saída já existe: {}",
            output.display()
        )));
    }
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let temporary = temporary_path(output);
    let result = (|| {
        let file = create_temporary_file(&temporary)?;
        let method = match compression {
            ContainerCompression::Deflate => CompressionMethod::Deflated,
            ContainerCompression::Store => CompressionMethod::Stored,
        };
        let compression_level = match compression {
            ContainerCompression::Deflate => Some(i32::from(level.numeric_hint())),
            ContainerCompression::Store => None,
        };
        let options = FileOptions::default()
            .compression_method(method)
            .compression_level(compression_level);
        let mut writer = ZipWriter::new(BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, file));
        let mut names = HashSet::new();
        let mut completed_bytes = 0_u64;
        let mut progress = |bytes: u64| {
            completed_bytes = completed_bytes.saturating_add(bytes);
            (callbacks.on_progress)(completed_bytes);
        };
        for input in &inputs {
            if is_cancelled() {
                return Err(CoreError::Cancelled);
            }
            append_input(
                &mut writer,
                input,
                options,
                &mut names,
                is_cancelled,
                &mut progress,
            )?;
        }
        let mut output_file = writer.finish().map_err(zip_error)?;
        output_file.flush()?;
        output_file.get_ref().sync_all()?;
        drop(output_file);
        (callbacks.on_validation_start)();
        let summary = validate_archive(&temporary)?;
        (callbacks.on_finalizing_start)();
        publish_file_without_overwrite(&temporary, output)?;
        Ok(summary)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn append_input<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    input: &InputEntry,
    options: FileOptions,
    names: &mut HashSet<String>,
    is_cancelled: &dyn Fn() -> bool,
    on_progress: &mut dyn FnMut(u64),
) -> CoreResult<()> {
    match input.kind {
        InputKind::File => {
            let name = input
                .path
                .file_name()
                .ok_or_else(|| CoreError::InvalidInput("arquivo sem nome".to_owned()))?;
            let relative = PathBuf::from(name);
            let key = normalized_name(&relative)?;
            if !names.insert(key.clone()) {
                return Err(CoreError::InvalidInput(format!(
                    "nomes de entrada duplicados no container: {key}"
                )));
            }
            writer.start_file(key, options).map_err(zip_error)?;
            let mut source =
                BufReader::with_capacity(DEFAULT_BUFFER_SIZE, File::open(&input.path)?);
            copy_with_cancel(&mut source, writer, is_cancelled, on_progress)?;
        }
        InputKind::Directory => {
            let root_name = input
                .path
                .file_name()
                .ok_or_else(|| CoreError::InvalidInput("diretório sem nome".to_owned()))?;
            append_directory(
                writer,
                &input.path,
                &PathBuf::from(root_name),
                options,
                names,
                is_cancelled,
                on_progress,
            )?;
        }
    }
    Ok(())
}

fn append_directory<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    directory: &Path,
    relative: &Path,
    options: FileOptions,
    names: &mut HashSet<String>,
    is_cancelled: &dyn Fn() -> bool,
    on_progress: &mut dyn FnMut(u64),
) -> CoreResult<()> {
    if is_cancelled() {
        return Err(CoreError::Cancelled);
    }
    let metadata = fs::symlink_metadata(directory)?;
    if is_link_or_reparse_point(&metadata) {
        return Err(CoreError::Unsupported(format!(
            "links simbólicos ou reparse points não são seguidos: {}",
            directory.display()
        )));
    }
    let key = format!("{}/", normalized_name(relative)?);
    if !names.insert(key.clone()) {
        return Err(CoreError::InvalidInput(format!(
            "nomes de entrada duplicados no container: {key}"
        )));
    }
    writer.add_directory(key, options).map_err(zip_error)?;
    for child in fs::read_dir(directory)? {
        let child = child?;
        let child_path = child.path();
        let metadata = fs::symlink_metadata(&child_path)?;
        let child_relative = relative.join(child.file_name());
        if is_link_or_reparse_point(&metadata) {
            return Err(CoreError::Unsupported(format!(
                "links simbólicos ou reparse points não são seguidos: {}",
                child_path.display()
            )));
        }
        if metadata.is_dir() {
            append_directory(
                writer,
                &child_path,
                &child_relative,
                options,
                names,
                is_cancelled,
                on_progress,
            )?;
        } else if metadata.is_file() {
            let key = normalized_name(&child_relative)?;
            if !names.insert(key.clone()) {
                return Err(CoreError::InvalidInput(format!(
                    "nomes de entrada duplicados no container: {key}"
                )));
            }
            writer.start_file(key, options).map_err(zip_error)?;
            let mut source =
                BufReader::with_capacity(DEFAULT_BUFFER_SIZE, File::open(&child_path)?);
            copy_with_cancel(&mut source, writer, is_cancelled, on_progress)?;
        } else {
            return Err(CoreError::Unsupported(format!(
                "tipo de entrada não suportado: {}",
                child_path.display()
            )));
        }
    }
    Ok(())
}

pub fn validate_archive(path: impl AsRef<Path>) -> CoreResult<ArchiveSummary> {
    let file = File::open(path)?;
    let mut archive =
        ZipArchive::new(BufReader::with_capacity(DEFAULT_BUFFER_SIZE, file)).map_err(zip_error)?;
    if archive.len() > MAX_ENTRIES {
        return Err(CoreError::InvalidInput(
            "quantidade de entradas excede o limite".to_owned(),
        ));
    }
    let mut entries = Vec::with_capacity(archive.len());
    let mut names = HashSet::new();
    let mut total_original_bytes = 0_u64;
    let mut total_compressed_bytes = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(zip_error)?;
        let relative = safe_zip_path(&entry)?;
        let name = normalized_name(&relative)?;
        if !names.insert(name.clone()) {
            return Err(CoreError::InvalidInput(format!(
                "entrada duplicada no container: {name}"
            )));
        }
        let (original_size, compressed_size, checksum) = validate_entry(&mut entry)?;
        total_original_bytes =
            total_original_bytes
                .checked_add(original_size)
                .ok_or_else(|| {
                    CoreError::InvalidInput("tamanho expandido excede o limite".to_owned())
                })?;
        if total_original_bytes > MAX_EXPANDED_BYTES {
            return Err(CoreError::InvalidInput(
                "tamanho expandido excede o limite de segurança".to_owned(),
            ));
        }
        total_compressed_bytes = total_compressed_bytes
            .checked_add(compressed_size)
            .ok_or_else(|| {
                CoreError::InvalidInput("tamanho comprimido excede o limite".to_owned())
            })?;
        entries.push(ArchiveEntry {
            path: relative,
            is_directory: entry.is_dir(),
            original_size,
            compressed_size,
            checksum,
            data_offset: entry.data_start(),
        });
    }
    Ok(ArchiveSummary {
        entries,
        total_original_bytes,
        total_compressed_bytes,
    })
}

pub fn extract_archive(
    path: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> CoreResult<ArchiveSummary> {
    let destination = destination.as_ref();
    if destination.exists() {
        return Err(CoreError::InvalidInput(format!(
            "o destino de extração já existe: {}",
            destination.display()
        )));
    }
    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let staging = temporary_path(destination);
    let result = (|| {
        fs::create_dir(&staging)?;
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(BufReader::with_capacity(DEFAULT_BUFFER_SIZE, file))
            .map_err(zip_error)?;
        if archive.len() > MAX_ENTRIES {
            return Err(CoreError::InvalidInput(
                "quantidade de entradas excede o limite".to_owned(),
            ));
        }
        let mut entries = Vec::with_capacity(archive.len());
        let mut names = HashSet::new();
        let mut total_original_bytes = 0_u64;
        let mut total_compressed_bytes = 0_u64;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(zip_error)?;
            let relative = safe_zip_path(&entry)?;
            let name = normalized_name(&relative)?;
            if !names.insert(name.clone()) {
                return Err(CoreError::InvalidInput(format!(
                    "entrada duplicada no container: {name}"
                )));
            }
            let target = staging.join(&relative);
            let (original_size, compressed_size, checksum) = if entry.is_dir() {
                if target.exists() && !target.is_dir() {
                    return Err(CoreError::InvalidInput(format!(
                        "conflito de caminho no container: {}",
                        relative.display()
                    )));
                }
                fs::create_dir_all(&target)?;
                (0, 0, 0)
            } else {
                if target.exists() {
                    return Err(CoreError::InvalidInput(format!(
                        "entrada duplicada no container: {}",
                        relative.display()
                    )));
                }
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                let temporary = temporary_path(&target);
                let result = extract_entry(&mut entry, &temporary, &target);
                if result.is_err() {
                    let _ = fs::remove_file(&temporary);
                }
                result?
            };
            let data_offset = entry.data_start();
            total_original_bytes =
                total_original_bytes
                    .checked_add(original_size)
                    .ok_or_else(|| {
                        CoreError::InvalidInput("tamanho expandido excede o limite".to_owned())
                    })?;
            if total_original_bytes > MAX_EXPANDED_BYTES {
                return Err(CoreError::InvalidInput(
                    "tamanho expandido excede o limite de segurança".to_owned(),
                ));
            }
            total_compressed_bytes = total_compressed_bytes
                .checked_add(compressed_size)
                .ok_or_else(|| {
                    CoreError::InvalidInput("tamanho comprimido excede o limite".to_owned())
                })?;
            entries.push(ArchiveEntry {
                path: relative,
                is_directory: entry.is_dir(),
                original_size,
                compressed_size,
                checksum,
                data_offset,
            });
        }
        fs::rename(&staging, destination)?;
        Ok(ArchiveSummary {
            entries,
            total_original_bytes,
            total_compressed_bytes,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn validate_entry(entry: &mut ZipFile<'_>) -> CoreResult<(u64, u64, u32)> {
    if entry.is_dir() {
        return Ok((0, 0, 0));
    }
    let mut hasher = Hasher::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; DEFAULT_BUFFER_SIZE];
    loop {
        let read = entry.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total = total.checked_add(read as u64).ok_or_else(|| {
            CoreError::InvalidInput("tamanho expandido excede o limite".to_owned())
        })?;
        if total > MAX_EXPANDED_BYTES {
            return Err(CoreError::InvalidInput(
                "tamanho expandido excede o limite de segurança".to_owned(),
            ));
        }
        ensure_expansion_ratio(entry.name(), total, entry.compressed_size())?;
    }
    let checksum = hasher.finalize();
    if checksum != entry.crc32() {
        return Err(CoreError::InvalidInput(format!(
            "falha de integridade no conteúdo: {}",
            entry.name()
        )));
    }
    let compressed_size = entry.compressed_size();
    ensure_expansion_ratio(entry.name(), total, compressed_size)?;
    Ok((total, compressed_size, checksum))
}

fn extract_entry(
    entry: &mut ZipFile<'_>,
    temporary: &Path,
    target: &Path,
) -> CoreResult<(u64, u64, u32)> {
    let mut output =
        BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, create_temporary_file(temporary)?);
    let mut hasher = Hasher::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; DEFAULT_BUFFER_SIZE];
    loop {
        let read = entry.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        total = total.checked_add(read as u64).ok_or_else(|| {
            CoreError::InvalidInput("tamanho expandido excede o limite".to_owned())
        })?;
        if total > MAX_EXPANDED_BYTES {
            return Err(CoreError::InvalidInput(
                "tamanho expandido excede o limite de segurança".to_owned(),
            ));
        }
        ensure_expansion_ratio(entry.name(), total, entry.compressed_size())?;
    }
    output.flush()?;
    output.get_ref().sync_all()?;
    drop(output);
    let checksum = hasher.finalize();
    if checksum != entry.crc32() {
        return Err(CoreError::InvalidInput(format!(
            "falha de integridade no conteúdo: {}",
            entry.name()
        )));
    }
    ensure_expansion_ratio(entry.name(), total, entry.compressed_size())?;
    publish_file_without_overwrite(temporary, target)?;
    Ok((total, entry.compressed_size(), checksum))
}

fn ensure_expansion_ratio(
    entry_name: &str,
    expanded_size: u64,
    compressed_size: u64,
) -> CoreResult<()> {
    if compressed_size > 0 && expanded_size > compressed_size.saturating_mul(MAX_COMPRESSION_RATIO)
    {
        return Err(CoreError::InvalidInput(format!(
            "razão de expansão excede o limite de segurança: {entry_name}"
        )));
    }
    Ok(())
}

fn safe_zip_path(entry: &ZipFile<'_>) -> CoreResult<PathBuf> {
    entry
        .enclosed_name()
        .map(safe_relative_path)
        .transpose()?
        .ok_or_else(|| {
            CoreError::InvalidInput(format!("caminho inseguro no container: {}", entry.name()))
        })
}

fn copy_with_cancel(
    input: &mut dyn Read,
    output: &mut dyn Write,
    is_cancelled: &dyn Fn() -> bool,
    on_progress: &mut dyn FnMut(u64),
) -> CoreResult<u64> {
    let mut buffer = [0_u8; DEFAULT_BUFFER_SIZE];
    let mut copied = 0_u64;
    loop {
        if is_cancelled() {
            return Err(CoreError::Cancelled);
        }
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(copied);
        }
        output.write_all(&buffer[..read])?;
        copied = copied.saturating_add(read as u64);
        on_progress(read as u64);
    }
}

fn ensure_output_does_not_overlap_inputs(inputs: &[InputEntry], output: &Path) -> CoreResult<()> {
    let output_absolute = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let output_path = normalize_nonexistent_path(&output_absolute)?;
    for input in inputs {
        let input_path = fs::canonicalize(&input.path)?;
        let overlaps = match input.kind {
            InputKind::File => input_path == output_path,
            InputKind::Directory => output_path.starts_with(&input_path),
        };
        if overlaps {
            return Err(CoreError::InvalidInput(
                "o arquivo de saída coincide ou está dentro de uma entrada".to_owned(),
            ));
        }
    }
    Ok(())
}

fn normalize_nonexistent_path(path: &Path) -> CoreResult<PathBuf> {
    let mut lexical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => lexical.push(prefix.as_os_str()),
            Component::RootDir => lexical.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                lexical.pop();
            }
            Component::Normal(part) => lexical.push(part),
        }
    }

    let mut missing = Vec::new();
    let mut current = lexical.as_path();
    while !current.exists() {
        let component = current.file_name().ok_or_else(|| {
            CoreError::InvalidInput(format!("caminho de saída inválido: {}", path.display()))
        })?;
        missing.push(PathBuf::from(component));
        current = current.parent().unwrap_or_else(|| Path::new("."));
    }
    let mut normalized = fs::canonicalize(current)?;
    for component in missing.iter().rev() {
        normalized.push(component);
    }
    Ok(normalized)
}

fn normalized_name(path: &Path) -> CoreResult<String> {
    let safe = safe_relative_path(path)?;
    let name = safe.to_str().ok_or_else(|| {
        CoreError::InvalidInput(format!(
            "nome de entrada não é Unicode válido: {}",
            path.display()
        ))
    })?;
    Ok(name.replace('\\', "/"))
}

fn create_temporary_file(path: &Path) -> CoreResult<File> {
    Ok(OpenOptions::new().write(true).create_new(true).open(path)?)
}

fn publish_file_without_overwrite(temporary: &Path, output: &Path) -> CoreResult<()> {
    match fs::hard_link(temporary, output) {
        Ok(()) => {
            let _ = fs::remove_file(temporary);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(CoreError::InvalidInput(format!(
                "o arquivo de saída já existe: {}",
                output.display()
            )))
        }
        Err(error) => Err(CoreError::Io(error)),
    }
}

fn temporary_path(target: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let mut temporary = target.to_path_buf();
    temporary.set_extension(format!("partial-{stamp}-{}", std::process::id()));
    temporary
}

fn zip_error(error: zip::result::ZipError) -> CoreError {
    match error {
        zip::result::ZipError::Io(error) => CoreError::Io(error),
        zip::result::ZipError::InvalidArchive(message) => {
            CoreError::InvalidInput(format!("container ZIP inválido: {message}"))
        }
        zip::result::ZipError::UnsupportedArchive(message) => {
            CoreError::Unsupported(format!("recurso do container ZIP não suportado: {message}"))
        }
        zip::result::ZipError::FileNotFound => {
            CoreError::InvalidInput("entrada não encontrada no container ZIP".to_owned())
        }
    }
}
