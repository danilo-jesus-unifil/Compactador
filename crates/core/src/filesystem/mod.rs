use crate::error::{CoreError, CoreResult};
use crate::models::{InputEntry, InputKind};
use std::fs;
use std::path::{Path, PathBuf};

pub fn inspect_input(path: impl AsRef<Path>) -> CoreResult<InputEntry> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(CoreError::from)?;
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
