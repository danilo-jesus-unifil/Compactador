use compactador_core::container::{
    compress_file, compress_inputs, compress_inputs_with_strategy, extract_archive,
    validate_archive, ContainerCompression,
};
use compactador_core::models::CompressionLevel;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

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
    assert!(summary.entries[0].data_offset > 0);
    assert!(validate_archive(&archive).is_ok());
    let extracted = extract_archive(&archive, &destination).expect("extract");
    assert_eq!(extracted.total_original_bytes, contents.len() as u64);
    assert!(extracted.entries[0].data_offset > 0);
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
fn directory_entries_are_emitted_in_sorted_order() {
    let root = temp_dir();
    let folder = root.join("Projeto Rust");
    fs::create_dir_all(&folder).expect("create folder");
    fs::write(folder.join("z.txt"), b"z").expect("write z");
    fs::write(folder.join("a.txt"), b"a").expect("write a");
    let archive_path = root.join("sorted.zip");
    compress_inputs(vec![folder], &archive_path, CompressionLevel::Normal)
        .expect("compress directory");
    let file = File::open(&archive_path).expect("open archive");
    let mut archive = ZipArchive::new(BufReader::new(file)).expect("read archive");
    let names = (0..archive.len())
        .map(|index| archive.by_index(index).expect("entry").name().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "Projeto Rust/".to_owned(),
            "Projeto Rust/a.txt".to_owned(),
            "Projeto Rust/z.txt".to_owned(),
        ]
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn rejects_case_insensitive_output_inside_input_directory() {
    let root = temp_dir();
    let input = root.join("Input");
    let output = root.join("input").join("nested.zip");
    fs::create_dir_all(&input).expect("create input");
    fs::write(input.join("arquivo.txt"), b"dados").expect("write input");
    assert!(compress_inputs(vec![input], &output, CompressionLevel::Normal).is_err());
    assert!(!output.exists());
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn rejects_symlink_inside_directory() {
    use std::os::unix::fs::symlink;

    let root = temp_dir();
    let input = root.join("input");
    let target = root.join("target.txt");
    let link = input.join("link.txt");
    fs::create_dir_all(&input).expect("create input");
    fs::write(&target, b"dados").expect("write target");
    symlink(&target, link).expect("create link");
    let output = root.join("output.zip");
    assert!(compress_inputs(vec![input], &output, CompressionLevel::Normal).is_err());
    assert!(!output.exists());
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

#[test]
fn applies_store_strategy_and_reports_progress() {
    use std::cell::Cell;

    let root = temp_dir();
    fs::create_dir_all(&root).expect("create root");
    let input = root.join("dados.zip");
    let archive_path = root.join("saida.zip");
    fs::write(&input, b"conteudo que nao deve ser recomprimido").expect("write input");
    let completed = Cell::new(0_u64);
    compress_inputs_with_strategy(
        vec![input],
        &archive_path,
        CompressionLevel::Normal,
        ContainerCompression::Store,
        &|| false,
        &|bytes| completed.set(bytes),
    )
    .expect("store compression");
    let file = File::open(&archive_path).expect("open archive");
    let mut archive = ZipArchive::new(BufReader::new(file)).expect("read archive");
    assert_eq!(
        archive.by_index(0).expect("entry").compression(),
        CompressionMethod::Stored
    );
    assert!(completed.get() > 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_case_insensitive_duplicate_archive_names() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("create root");
    let archive = root.join("case-duplicate.zip");
    let file = File::create(&archive).expect("create archive");
    let mut writer = ZipWriter::new(file);
    writer
        .start_file("Foo.txt", FileOptions::default())
        .expect("start first");
    writer.write_all(b"first").expect("write first");
    writer
        .start_file("foo.txt", FileOptions::default())
        .expect("start second");
    writer.write_all(b"second").expect("write second");
    writer.finish().expect("finish archive");
    assert!(validate_archive(&archive).is_err());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_output_inside_input_directory() {
    let root = temp_dir();
    let input = root.join("pasta");
    fs::create_dir_all(&input).expect("create input");
    fs::write(input.join("arquivo.txt"), b"dados").expect("write input");
    let output = input.join("saida.zip");
    assert!(compress_inputs(vec![input], &output, CompressionLevel::Normal).is_err());
    assert!(!output.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn extraction_rejects_traversal_and_existing_destination() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("create root");
    let destination = root.join("destino");
    for (index, name) in [
        "../escape.txt",
        "C:\\escape.txt",
        "\\\\server\\share\\escape.txt",
        "pasta\\..\\escape.txt",
        "CON.txt",
        "NUL",
        "arquivo?.txt",
        "arquivo.txt ",
        "arquivo.txt.",
    ]
    .into_iter()
    .enumerate()
    {
        let malicious = root.join(format!("malicious-{index}.zip"));
        let file = File::create(&malicious).expect("create archive");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file(name, FileOptions::default())
            .expect("start malicious entry");
        writer.write_all(b"escape").expect("write malicious entry");
        writer.finish().expect("finish archive");
        assert!(extract_archive(&malicious, &destination).is_err());
        assert!(!destination.exists());
    }

    let input = root.join("arquivo.txt");
    let valid = root.join("valid.zip");
    fs::write(&input, b"dados").expect("write valid input");
    compress_file(&input, &valid, CompressionLevel::Normal).expect("compress valid");
    fs::create_dir_all(&destination).expect("create existing destination");
    fs::write(destination.join("sentinela.txt"), b"preservar").expect("write sentinel");
    assert!(extract_archive(&valid, &destination).is_err());
    assert_eq!(
        fs::read(destination.join("sentinela.txt")).expect("read sentinel"),
        b"preservar"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn extraction_rejects_duplicate_directory_entries() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("create root");
    let archive = root.join("duplicate-directories.zip");
    let file = File::create(&archive).expect("create archive");
    let mut writer = ZipWriter::new(file);
    writer
        .add_directory("folder/", FileOptions::default())
        .expect("start first directory");
    writer
        .add_directory("folder/", FileOptions::default())
        .expect("start second directory");
    writer.finish().expect("finish archive");
    let destination = root.join("destination");
    assert!(extract_archive(&archive, &destination).is_err());
    assert!(!destination.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_duplicate_archive_names() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("create root");
    let archive = root.join("duplicate.zip");
    let file = File::create(&archive).expect("create archive");
    let mut writer = ZipWriter::new(file);
    writer
        .start_file("same.txt", FileOptions::default())
        .expect("start first");
    writer.write_all(b"first").expect("write first");
    writer
        .start_file("same.txt", FileOptions::default())
        .expect("start second");
    writer.write_all(b"second").expect("write second");
    writer.finish().expect("finish archive");
    assert!(validate_archive(&archive).is_err());
    let _ = fs::remove_dir_all(root);
}
