use crate::error::{CoreError, CoreResult};
use std::path::{Component, Path, PathBuf};

pub fn safe_relative_path(path: &Path) -> CoreResult<PathBuf> {
    if path.is_absolute() {
        return Err(CoreError::InvalidInput(
            "caminho armazenado deve ser relativo".to_owned(),
        ));
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
    use std::path::Path;

    #[test]
    fn rejects_parent_traversal() {
        assert!(safe_relative_path(Path::new("../arquivo.txt")).is_err());
    }

    #[test]
    fn accepts_nested_relative_path() {
        let result = safe_relative_path(Path::new("pasta/arquivo.txt"));
        assert_eq!(result.ok().as_deref(), Some(Path::new("pasta/arquivo.txt")));
    }
}
