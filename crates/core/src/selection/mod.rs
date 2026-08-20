use crate::error::{CoreError, CoreResult};
use crate::filesystem::validate_inputs;
use crate::models::{
    CompressionLevel, CompressionStrategy, FileClassification, InputEntry, ResourceProfile,
};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputProfile {
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub directory_count: u64,
    pub has_compressed_content: bool,
    pub dominant_category: Option<FileClassification>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRequest {
    pub level: CompressionLevel,
    pub inputs: Vec<InputEntry>,
    pub output: Option<PathBuf>,
}

impl SelectionRequest {
    pub fn parse<I>(arguments: I) -> CoreResult<Self>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut level = None;
        let mut output = None;
        let mut paths = Vec::new();
        let mut after_separator = false;
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            if !after_separator && argument == "--" {
                after_separator = true;
                continue;
            }
            if !after_separator && argument == "--compress" {
                let value = arguments.next().ok_or_else(|| {
                    CoreError::InvalidInput("--compress exige um nível".to_owned())
                })?;
                level = Some(parse_level(&value)?);
                continue;
            }
            if !after_separator && argument == "--output" {
                let value = arguments.next().ok_or_else(|| {
                    CoreError::InvalidInput("--output exige um caminho".to_owned())
                })?;
                output = Some(PathBuf::from(value));
                continue;
            }
            if !after_separator && argument.to_string_lossy().starts_with('-') {
                return Err(CoreError::InvalidInput(format!(
                    "opção desconhecida: {}",
                    argument.to_string_lossy()
                )));
            }
            paths.push(PathBuf::from(argument));
        }
        let level = level
            .ok_or_else(|| CoreError::InvalidInput("nível de compactação ausente".to_owned()))?;
        let inputs = validate_inputs(&paths)?;
        Ok(Self {
            level,
            inputs,
            output,
        })
    }
}

pub trait StrategySelector {
    fn select(
        &self,
        profile: &InputProfile,
        level: CompressionLevel,
        resources: &ResourceProfile,
    ) -> CoreResult<CompressionStrategy>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HeuristicStrategySelector;

impl StrategySelector for HeuristicStrategySelector {
    fn select(
        &self,
        profile: &InputProfile,
        level: CompressionLevel,
        resources: &ResourceProfile,
    ) -> CoreResult<CompressionStrategy> {
        if profile.file_count == 0 && profile.directory_count == 0 {
            return Err(CoreError::InvalidInput(
                "o seletor recebeu uma seleção sem arquivos ou diretórios".to_owned(),
            ));
        }
        let category = profile
            .dominant_category
            .unwrap_or(FileClassification::Unknown);
        let already_compressed = profile.has_compressed_content
            || matches!(
                category,
                FileClassification::Archive
                    | FileClassification::CompressedImage
                    | FileClassification::CompressedAudio
                    | FileClassification::Video
            );
        let parallel = false;
        let (algorithm_id, rationale) = if already_compressed {
            ("store", format!("conteúdo predominantemente já comprimido; armazenamento direto evita custo adicional no nível {level}"))
        } else if level == CompressionLevel::Maximum
            && resources.available_memory_bytes >= 256 * 1024 * 1024
        {
            (
                "deflate",
                format!("nível {level} prioriza redução de tamanho para a categoria {category:?}"),
            )
        } else {
            (
                "deflate",
                format!("categoria {category:?}, nível {level}, custo de CPU equilibrado"),
            )
        };
        let estimated_gain_percent = if already_compressed {
            5
        } else {
            match level {
                CompressionLevel::Fast => 25,
                CompressionLevel::Low => 35,
                CompressionLevel::Normal => 50,
                CompressionLevel::High => 60,
                CompressionLevel::Maximum => 65,
            }
        };
        Ok(CompressionStrategy {
            algorithm_id: algorithm_id.to_owned(),
            level,
            rationale,
            estimated_gain_percent,
            parallel,
        })
    }
}

fn parse_level(value: &OsString) -> CoreResult<CompressionLevel> {
    if let Some(raw) = value.to_str() {
        if let Ok(level) = raw.parse::<CompressionLevel>() {
            return Ok(level);
        }
    }
    match value.to_string_lossy().as_ref() {
        "1" => Ok(CompressionLevel::Fast),
        "3" => Ok(CompressionLevel::Low),
        "6" => Ok(CompressionLevel::Normal),
        "8" => Ok(CompressionLevel::High),
        "9" => Ok(CompressionLevel::Maximum),
        _ => Err(CoreError::InvalidInput(format!(
            "nível de compactação inválido: {}",
            value.to_string_lossy()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_multiple_unicode_paths_without_splitting_spaces() {
        let root = std::env::temp_dir().join(format!(
            "compactador-selection-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let first = root.join("Meu arquivo.txt");
        let second = root.join("ação (backup).txt");
        fs::write(&first, b"one").expect("write first");
        fs::write(&second, b"two").expect("write second");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--output"),
            root.join("out.zip").as_os_str().to_os_string(),
            OsString::from("--"),
            first.as_os_str().to_os_string(),
            second.as_os_str().to_os_string(),
        ])
        .expect("parse selection");
        assert_eq!(request.level, CompressionLevel::Normal);
        assert_eq!(request.inputs.len(), 2);
        assert_eq!(request.output, Some(root.join("out.zip")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn selects_conservative_strategy_for_already_compressed_content() {
        let selector = HeuristicStrategySelector;
        let profile = InputProfile {
            total_size_bytes: 1,
            file_count: 1,
            directory_count: 0,
            has_compressed_content: true,
            dominant_category: Some(FileClassification::Archive),
        };
        let strategy = selector
            .select(
                &profile,
                CompressionLevel::Maximum,
                &ResourceProfile::default(),
            )
            .expect("strategy");
        assert_eq!(strategy.algorithm_id, "store");
        assert!(!strategy.parallel);
    }

    #[test]
    fn rejects_missing_level_and_empty_selection() {
        assert!(
            SelectionRequest::parse([OsString::from("--compress"), OsString::from("normal")])
                .is_err()
        );
        assert!(SelectionRequest::parse([OsString::from("--"), OsString::from("x")]).is_err());
    }
}
