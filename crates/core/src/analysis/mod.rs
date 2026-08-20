use crate::filesystem::is_link_or_reparse_point;
use crate::models::{Classification, Confidence, FileClassification, InputEntry, InputKind};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub const DEFAULT_SAMPLE_BYTES: usize = 64 * 1024;
pub const MAX_DIRECTORY_ANALYSIS_FILES: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub struct FileAnalysis {
    pub path: std::path::PathBuf,
    pub size_bytes: u64,
    pub classification: Classification,
    pub estimated_compressibility_percent: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionAnalysis {
    pub files: u64,
    pub directories: u64,
    pub total_size_bytes: u64,
    pub estimated_compressibility_percent: u8,
    pub dominant_category: Option<FileClassification>,
    pub already_compressed: bool,
    pub sampled: bool,
}

pub trait ContentClassifier: Send + Sync {
    fn classify(&self, path: &Path) -> io::Result<Classification>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExtensionClassifier;

impl ContentClassifier for ExtensionClassifier {
    fn classify(&self, path: &Path) -> io::Result<Classification> {
        Ok(extension_classification(path))
    }
}

pub fn extension_classification(path: &Path) -> Classification {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let kind = match extension.as_str() {
        "txt" | "md" | "log" | "ini" | "cfg" => FileClassification::Text,
        "rs" | "toml" | "c" | "cpp" | "h" | "py" | "js" | "ts" | "java" | "go" => {
            FileClassification::SourceCode
        }
        "json" | "xml" | "csv" | "yaml" | "yml" => FileClassification::StructuredData,
        "jpg" | "jpeg" | "png" | "webp" | "gif" => FileClassification::CompressedImage,
        "bmp" | "tif" | "tiff" | "ppm" => FileClassification::CompressibleImage,
        "wav" | "flac" | "aiff" => FileClassification::UncompressedAudio,
        "mp3" | "aac" | "ogg" | "m4a" => FileClassification::CompressedAudio,
        "mp4" | "mkv" | "avi" | "mov" | "webm" => FileClassification::Video,
        "zip" | "7z" | "rar" | "gz" | "bz2" | "xz" | "zst" => FileClassification::Archive,
        "exe" | "dll" | "so" | "bin" | "o" => FileClassification::Executable,
        "db" | "sqlite" | "sqlite3" => FileClassification::Database,
        "pdf" | "doc" | "docx" | "odt" => FileClassification::Document,
        _ => FileClassification::Unknown,
    };
    let confidence = if kind == FileClassification::Unknown {
        Confidence::Low
    } else {
        Confidence::Medium
    };
    Classification { kind, confidence }
}

pub fn analyze_file(path: impl AsRef<Path>) -> io::Result<FileAnalysis> {
    let path = path.as_ref();
    let metadata = std::fs::symlink_metadata(path)?;
    if is_link_or_reparse_point(&metadata) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "links simbólicos ou reparse points não são analisados: {}",
                path.display()
            ),
        ));
    }
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("não é arquivo regular: {}", path.display()),
        ));
    }
    let mut classification = extension_classification(path);
    let mut sample = Vec::new();
    let mut reader = File::open(path)?.take(DEFAULT_SAMPLE_BYTES as u64);
    reader.read_to_end(&mut sample)?;
    if classification.kind == FileClassification::Unknown {
        classification = classify_sample(&sample);
    }
    let estimated = estimate_compressibility(&sample, classification.kind);
    let rationale = format!(
        "tipo {:?}, confiança {:?}, amostra de {} bytes",
        classification.kind,
        classification.confidence,
        sample.len()
    );
    Ok(FileAnalysis {
        path: path.to_path_buf(),
        size_bytes: metadata.len(),
        classification,
        estimated_compressibility_percent: estimated,
        rationale,
    })
}

pub fn analyze_selection(inputs: &[InputEntry]) -> io::Result<SelectionAnalysis> {
    let mut accumulator = AnalysisAccumulator {
        already_compressed: true,
        ..AnalysisAccumulator::default()
    };
    for input in inputs {
        match input.kind {
            InputKind::File => accumulator.add_file(&input.path)?,
            InputKind::Directory => {
                accumulator.directories += 1;
                analyze_directory(&input.path, &mut accumulator)?;
            }
        }
    }
    Ok(accumulator.finish())
}

#[derive(Default)]
struct AnalysisAccumulator {
    files: u64,
    directories: u64,
    total_size_bytes: u64,
    analyzed_size_bytes: u64,
    weighted_compressibility: u64,
    category_sizes: std::collections::HashMap<FileClassification, u64>,
    already_compressed: bool,
    sampled: bool,
}

impl AnalysisAccumulator {
    fn add_file(&mut self, path: &Path) -> io::Result<()> {
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_file() {
            return Ok(());
        }
        self.total_size_bytes = self.total_size_bytes.saturating_add(metadata.len());
        if self.files as usize >= MAX_DIRECTORY_ANALYSIS_FILES {
            self.sampled = true;
            return Ok(());
        }
        let analysis = analyze_file(path)?;
        self.files += 1;
        self.analyzed_size_bytes = self.analyzed_size_bytes.saturating_add(analysis.size_bytes);
        self.weighted_compressibility = self.weighted_compressibility.saturating_add(
            analysis
                .size_bytes
                .saturating_mul(u64::from(analysis.estimated_compressibility_percent)),
        );
        let category_size = self
            .category_sizes
            .entry(analysis.classification.kind)
            .or_insert(0);
        *category_size = category_size.saturating_add(analysis.size_bytes);
        if !matches!(
            analysis.classification.kind,
            FileClassification::Archive
                | FileClassification::CompressedImage
                | FileClassification::CompressedAudio
                | FileClassification::Video
        ) {
            self.already_compressed = false;
        }
        Ok(())
    }

    fn finish(self) -> SelectionAnalysis {
        let dominant_category = self
            .category_sizes
            .into_iter()
            .max_by_key(|(_, size)| *size)
            .map(|(category, _)| category);
        let estimated = self
            .weighted_compressibility
            .checked_div(self.analyzed_size_bytes)
            .unwrap_or(0)
            .min(100) as u8;
        SelectionAnalysis {
            files: self.files,
            directories: self.directories,
            total_size_bytes: self.total_size_bytes,
            estimated_compressibility_percent: estimated,
            dominant_category,
            already_compressed: self.files > 0 && self.already_compressed && !self.sampled,
            sampled: self.sampled,
        }
    }
}

fn analyze_directory(path: &Path, accumulator: &mut AnalysisAccumulator) -> io::Result<()> {
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&directory)?;
        if is_link_or_reparse_point(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "links simbólicos ou reparse points não são analisados: {}",
                    directory.display()
                ),
            ));
        }
        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("não é diretório regular: {}", directory.display()),
            ));
        }
        for child in std::fs::read_dir(&directory)? {
            let child = child?;
            let child_path = child.path();
            let metadata = std::fs::symlink_metadata(&child_path)?;
            if is_link_or_reparse_point(&metadata) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!(
                        "links simbólicos ou reparse points não são analisados: {}",
                        child_path.display()
                    ),
                ));
            } else if metadata.is_dir() {
                pending.push(child_path);
            } else if metadata.is_file() {
                accumulator.add_file(&child_path)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!(
                        "tipo de entrada não suportado na análise: {}",
                        child_path.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn classify_sample(sample: &[u8]) -> Classification {
    if sample.is_empty() {
        return Classification {
            kind: FileClassification::Unknown,
            confidence: Confidence::Low,
        };
    }
    let printable = sample
        .iter()
        .filter(|byte| matches!(byte, 9 | 10 | 13 | 32..=126))
        .count();
    let ratio = printable as f32 / sample.len() as f32;
    let kind = if ratio > 0.85 {
        FileClassification::Text
    } else {
        FileClassification::Unknown
    };
    let confidence = if ratio > 0.95 {
        Confidence::Medium
    } else {
        Confidence::Low
    };
    Classification { kind, confidence }
}

fn estimate_compressibility(sample: &[u8], classification: FileClassification) -> u8 {
    if matches!(
        classification,
        FileClassification::Archive
            | FileClassification::CompressedImage
            | FileClassification::CompressedAudio
            | FileClassification::Video
    ) {
        return 5;
    }
    if sample.is_empty() {
        return 30;
    }
    let unique = sample.iter().copied().collect::<HashSet<_>>().len();
    let diversity = unique as f32 / 256.0;
    let base = ((1.0 - diversity) * 100.0).round() as u8;
    base.clamp(10, 95)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn classifies_known_extensions_and_unknown_sample() {
        assert_eq!(
            extension_classification(Path::new("dados.json")).kind,
            FileClassification::StructuredData
        );
        let root = std::env::temp_dir().join(format!(
            "compactador-analysis-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let path = root.join("desconhecido.binx");
        fs::write(&path, b"texto repetido texto repetido texto repetido").expect("write sample");
        let analysis = analyze_file(&path).expect("analysis");
        assert_eq!(analysis.classification.kind, FileClassification::Text);
        assert!(analysis.estimated_compressibility_percent > 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sampled_profile_does_not_claim_all_content_compressed() {
        let profile = AnalysisAccumulator {
            files: 1,
            total_size_bytes: 10,
            analyzed_size_bytes: 5,
            weighted_compressibility: 25,
            already_compressed: true,
            sampled: true,
            ..AnalysisAccumulator::default()
        }
        .finish();
        assert!(profile.sampled);
        assert!(!profile.already_compressed);
        assert_eq!(profile.estimated_compressibility_percent, 5);
    }

    #[test]
    fn marks_known_compressed_content_in_selection_analysis() {
        let root = std::env::temp_dir().join(format!(
            "compactador-analysis-archive-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let path = root.join("dados.zip");
        fs::write(&path, b"conteudo de teste").expect("write archive placeholder");
        let input = InputEntry::new(path.clone(), InputKind::File, Some(18));
        let analysis = analyze_selection(&[input]).expect("selection analysis");
        assert!(analysis.already_compressed);
        let _ = fs::remove_dir_all(root);
    }
}
