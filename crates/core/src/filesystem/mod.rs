use crate::error::{CoreError, CoreResult};
use crate::models::{InputEntry, InputKind};
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};

pub fn inspect_input(path: impl AsRef<Path>) -> CoreResult<InputEntry> {
    let path = path.as_ref();
    let metadata = fs::symlink_metadata(path).map_err(CoreError::from)?;
    if is_link_or_reparse_point(&metadata) {
        return Err(CoreError::Unsupported(format!(
            "links simbólicos ou reparse points não são aceitos como entrada: {}",
            path.display()
        )));
    }
    let kind = if metadata.is_dir() {
        InputKind::Directory
    } else if metadata.is_file() {
        InputKind::File
    } else {
        return Err(CoreError::InvalidInput(format!(
            "tipo de caminho não suportado: {}",
            path.display()
        )));
    };
    let size_bytes = metadata.is_file().then_some(metadata.len());
    Ok(InputEntry::new(path.to_path_buf(), kind, size_bytes))
}

pub fn validate_inputs(paths: &[PathBuf]) -> CoreResult<Vec<InputEntry>> {
    if paths.is_empty() {
        return Err(CoreError::InvalidInput(
            "nenhuma entrada foi fornecida".to_owned(),
        ));
    }
    paths.iter().map(inspect_input).collect()
}

#[cfg(windows)]
pub fn is_link_or_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub fn is_link_or_reparse_point(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_regular_file_and_directory() {
        let root = std::env::temp_dir().join(format!(
            "compactador-filesystem-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let file = root.join("arquivo.txt");
        fs::write(&file, b"dados").expect("write file");
        assert_eq!(
            inspect_input(&file).expect("inspect file").kind,
            InputKind::File
        );
        assert_eq!(
            inspect_input(&root).expect("inspect directory").kind,
            InputKind::Directory
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_root_symlink() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "compactador-filesystem-link-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let target = root.join("target.txt");
        let link = root.join("link.txt");
        fs::write(&target, b"dados").expect("write target");
        symlink(&target, &link).expect("create link");
        assert!(matches!(
            inspect_input(&link),
            Err(CoreError::Unsupported(_))
        ));
        let _ = fs::remove_dir_all(root);
    }
}
