use crate::error::{CoreError, CoreResult};
use std::path::{Component, Path, PathBuf};

pub fn safe_relative_path(path: &Path) -> CoreResult<PathBuf> {
    let raw = path.to_str().ok_or_else(|| {
        CoreError::InvalidInput(format!(
            "caminho armazenado não é Unicode válido: {}",
            path.display()
        ))
    })?;
    if raw.chars().any(|character| character.is_control())
        || path.is_absolute()
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
            Component::Normal(part) => {
                validate_windows_component(part, path)?;
                safe.push(part);
            }
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

fn validate_windows_component(component: &std::ffi::OsStr, path: &Path) -> CoreResult<()> {
    let value = component.to_str().ok_or_else(|| {
        CoreError::InvalidInput(format!(
            "componente de caminho não é Unicode válido: {}",
            path.display()
        ))
    })?;
    if value.is_empty()
        || value.ends_with(['.', ' '])
        || value
            .chars()
            .any(|character| matches!(character, '<' | '>' | '"' | '|' | '?' | '*'))
    {
        return Err(CoreError::InvalidInput(format!(
            "componente de caminho incompatível com Windows: {}",
            path.display()
        )));
    }
    let stem = value.split('.').next().unwrap_or_default();
    if matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "COM¹"
            | "COM²"
            | "COM³"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
            | "LPT¹"
            | "LPT²"
            | "LPT³"
    ) {
        return Err(CoreError::InvalidInput(format!(
            "componente de caminho reservado pelo Windows: {}",
            path.display()
        )));
    }
    Ok(())
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
            "arquivo\0invalido.txt",
            "CON.txt",
            "COM¹.txt",
            "LPT³",
            "NUL",
            "arquivo?.txt",
            "arquivo.txt ",
            "arquivo.txt.",
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
