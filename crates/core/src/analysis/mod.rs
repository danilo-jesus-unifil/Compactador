use crate::models::{Classification, Confidence, FileClassification};
use std::path::Path;

pub trait ContentClassifier: Send + Sync {
    fn classify(&self, path: &Path) -> std::io::Result<Classification>;
}

pub fn extension_classification(path: &Path) -> Classification {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let kind = match extension.as_str() {
        "txt" | "md" | "log" => FileClassification::Text,
        "rs" | "toml" | "c" | "cpp" | "h" | "py" | "js" | "ts" => FileClassification::SourceCode,
        "json" | "xml" | "csv" | "yaml" | "yml" => FileClassification::StructuredData,
        "jpg" | "jpeg" | "png" | "webp" | "gif" => FileClassification::CompressedImage,
        "bmp" | "tif" | "tiff" => FileClassification::CompressibleImage,
        "wav" | "flac" => FileClassification::UncompressedAudio,
        "mp3" | "aac" | "ogg" | "m4a" => FileClassification::CompressedAudio,
        "mp4" | "mkv" | "avi" | "mov" | "webm" => FileClassification::Video,
        "zip" | "7z" | "rar" | "gz" | "bz2" | "xz" => FileClassification::Archive,
        "exe" | "dll" | "so" | "bin" => FileClassification::Executable,
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
