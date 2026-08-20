use crate::error::{CoreError, CoreResult};
use crate::filesystem::validate_inputs;
use crate::models::{CompressionLevel, InputEntry, InputKind};
use crate::security::safe_relative_path;
use crc32fast::Hasher;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::read::ZipFile;
use zip::write::{FileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

pub const DEFAULT_BUFFER_SIZE: usize = 128 * 1024;
pub const MAX_ENTRIES: usize = 1_000_000;
pub const MAX_EXPANDED_BYTES: u64 = 1_u64 << 50;

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
    let inputs = validate_inputs(paths.as_ref())?;
    let output = output.as_ref();
    if inputs.iter().any(|entry| entry.path == output) {
        return Err(CoreError::InvalidInput(
            "o arquivo de saída não pode ser uma entrada".to_owned(),
        ));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = temporary_path(output);
    let result = (|| {
        let file = File::create(&temporary)?;
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(i32::from(level.numeric_hint())));
        let mut writer = ZipWriter::new(BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, file));
        let mut names = HashSet::new();
        let mut entries = Vec::new();
        for input in &inputs {
            if is_cancelled() {
                return Err(CoreError::Cancelled);
            }
            append_input(
                &mut writer,
                input,
                options,
                &mut names,
                &mut entries,
                is_cancelled,
            )?;
        }
        let mut output_file = writer.finish().map_err(zip_error)?;
        output_file.flush()?;
        output_file.get_ref().sync_all()?;
        drop(output_file);
        let summary = validate_archive(&temporary)?;
        fs::rename(&temporary, output)?;
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
    entries: &mut Vec<ArchiveEntry>,
    is_cancelled: &dyn Fn() -> bool,
) -> CoreResult<()> {
    match input.kind {
        InputKind::File => {
            let name = input
                .path
                .file_name()
                .ok_or_else(|| CoreError::InvalidInput("arquivo sem nome".to_owned()))?;
            let relative = PathBuf::from(name);
            let key = normalized_name(&relative);
            if !names.insert(key.clone()) {
                return Err(CoreError::InvalidInput(format!(
                    "nomes de entrada duplicados no container: {key}"
                )));
            }
            writer.start_file(key, options).map_err(zip_error)?;
            let mut source =
                BufReader::with_capacity(DEFAULT_BUFFER_SIZE, File::open(&input.path)?);
            copy_with_cancel(&mut source, writer, is_cancelled)?;
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
                entries,
                is_cancelled,
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
    entries: &mut Vec<ArchiveEntry>,
    is_cancelled: &dyn Fn() -> bool,
) -> CoreResult<()> {
    if is_cancelled() {
        return Err(CoreError::Cancelled);
    }
    let key = format!("{}/", normalized_name(relative));
    if !names.insert(key.clone()) {
        return Err(CoreError::InvalidInput(format!(
            "nomes de entrada duplicados no container: {key}"
        )));
    }
    writer.add_directory(key, options).map_err(zip_error)?;
    entries.push(ArchiveEntry {
        path: relative.to_path_buf(),
        is_directory: true,
        original_size: 0,
        compressed_size: 0,
        checksum: 0,
        data_offset: 0,
    });
    let mut children = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let child_path = child.path();
        let metadata = fs::symlink_metadata(&child_path)?;
        let child_relative = relative.join(child.file_name());
        if metadata.file_type().is_symlink() {
            return Err(CoreError::Unsupported(format!(
                "links simbólicos não são seguidos: {}",
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
                entries,
                is_cancelled,
            )?;
        } else if metadata.is_file() {
            let key = normalized_name(&child_relative);
            if !names.insert(key.clone()) {
                return Err(CoreError::InvalidInput(format!(
                    "nomes de entrada duplicados no container: {key}"
                )));
            }
            writer.start_file(key, options).map_err(zip_error)?;
            let mut source =
                BufReader::with_capacity(DEFAULT_BUFFER_SIZE, File::open(&child_path)?);
            copy_with_cancel(&mut source, writer, is_cancelled)?;
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
    let mut total_original_bytes = 0_u64;
    let mut total_compressed_bytes = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(zip_error)?;
        let relative = safe_zip_path(&entry)?;
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
            data_offset: 0,
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
    fs::create_dir_all(destination)?;
    let file = File::open(path)?;
    let mut archive =
        ZipArchive::new(BufReader::with_capacity(DEFAULT_BUFFER_SIZE, file)).map_err(zip_error)?;
    if archive.len() > MAX_ENTRIES {
        return Err(CoreError::InvalidInput(
            "quantidade de entradas excede o limite".to_owned(),
        ));
    }
    let mut entries = Vec::with_capacity(archive.len());
    let mut total_original_bytes = 0_u64;
    let mut total_compressed_bytes = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(zip_error)?;
        let relative = safe_zip_path(&entry)?;
        let target = destination.join(&relative);
        let (original_size, compressed_size, checksum) = if entry.is_dir() {
            fs::create_dir_all(&target)?;
            (0, 0, 0)
        } else {
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
            data_offset: 0,
        });
    }
    Ok(ArchiveSummary {
        entries,
        total_original_bytes,
        total_compressed_bytes,
    })
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
    }
    let checksum = hasher.finalize();
    if checksum != entry.crc32() {
        return Err(CoreError::InvalidInput(format!(
            "falha de integridade no conteúdo: {}",
            entry.name()
        )));
    }
    Ok((total, entry.compressed_size(), checksum))
}

fn extract_entry(
    entry: &mut ZipFile<'_>,
    temporary: &Path,
    target: &Path,
) -> CoreResult<(u64, u64, u32)> {
    let mut output = BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, File::create(temporary)?);
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
    fs::rename(temporary, target)?;
    Ok((total, entry.compressed_size(), checksum))
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
    }
}

fn normalized_name(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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
    CoreError::InvalidInput(format!("container ZIP inválido: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("compactador-test-{stamp}"))
    }

    #[test]
    fn compresses_validates_and_extracts_unicode_file() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("Meu arquivo.txt");
        let archive = root.join("saida.zip");
        let destination = root.join("out");
        let contents = "Rust e caminhos Unicode: ação, seleção e dados repetidos. ".repeat(256);
        fs::write(&input, contents.as_bytes()).expect("write input");
        let summary = compress_file(&input, &archive, CompressionLevel::Normal).expect("compress");
        assert_eq!(summary.entries.len(), 1);
        assert!(validate_archive(&archive).is_ok());
        let extracted = extract_archive(&archive, &destination).expect("extract");
        assert_eq!(extracted.total_original_bytes, contents.len() as u64);
        assert_eq!(
            fs::read(destination.join("Meu arquivo.txt")).expect("read output"),
            contents.as_bytes()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn supports_directory_and_multiple_selection_without_following_symlinks() {
        let root = temp_dir();
        let folder = root.join("Projeto Rust");
        fs::create_dir_all(folder.join("src")).expect("create folder");
        fs::write(folder.join("README.md"), b"readme").expect("write readme");
        fs::write(folder.join("src/main.rs"), b"fn main() {}").expect("write source");
        let second = root.join("dados.csv");
        fs::write(&second, b"a,b\n1,2\n").expect("write csv");
        let archive = root.join("multi.zip");
        let summary = compress_inputs(
            vec![folder.clone(), second.clone()],
            &archive,
            CompressionLevel::Fast,
        )
        .expect("compress selection");
        assert!(summary.entries.len() >= 4);
        assert!(validate_archive(&archive).is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_archive() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create root");
        let path = root.join("bad.zip");
        fs::write(&path, b"not-a-container").expect("write bad archive");
        assert!(validate_archive(&path).is_err());
        let _ = fs::remove_dir_all(root);
    }
}
