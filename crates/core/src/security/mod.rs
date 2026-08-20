use crate::error::{CoreError, CoreResult};
use std::path::{Component, Path, PathBuf};

pub fn safe_relative_path(path: &Path) -> CoreResult<PathBuf> {
    let raw = path.to_string_lossy();
    if path.is_absolute()
        || raw.starts_with('/')
        || raw.starts_with('\\')
        || raw.contains(':')
        || raw.contains('\\')
    {
        return Err(CoreError::InvalidInput(format!(
            "caminho armazenado deve ser relativo e portátil: {}",
            path.display()
        )));
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CoreError::InvalidInput(format!(
                    "caminho relativo inseguro: {}",
                    path.display()
                )));
            }
        }
    }
    if safe.as_os_str().is_empty() {
        return Err(CoreError::InvalidInput("caminho relativo vazio".to_owned()));
    }
    Ok(safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_traversal_and_windows_absolute_forms() {
        for path in [
            "../arquivo.txt",
            "C:\\arquivo.txt",
            "\\\\servidor\\share\\arquivo.txt",
            "pasta\\..\\arquivo.txt",
        ] {
            assert!(
                safe_relative_path(Path::new(path)).is_err(),
                "unsafe path accepted: {path}"
            );
        }
    }

    #[test]
    fn accepts_nested_portable_relative_path() {
        let result = safe_relative_path(Path::new("pasta/arquivo.txt"));
        assert_eq!(result.ok().as_deref(), Some(Path::new("pasta/arquivo.txt")));
    }
}
