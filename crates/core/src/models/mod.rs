use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    Fast,
    Low,
    Normal,
    High,
    Maximum,
}

impl CompressionLevel {
    pub const ALL: [Self; 5] = [
        Self::Fast,
        Self::Low,
        Self::Normal,
        Self::High,
        Self::Maximum,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Fast => "Rápida",
            Self::Low => "Baixa",
            Self::Normal => "Normal",
            Self::High => "Alta",
            Self::Maximum => "Máxima",
        }
    }

    pub const fn numeric_hint(self) -> u8 {
        match self {
            Self::Fast => 1,
            Self::Low => 3,
            Self::Normal => 6,
            Self::High => 8,
            Self::Maximum => 9,
        }
    }
}

impl fmt::Display for CompressionLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display_name())
    }
}

impl FromStr for CompressionLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "fast" | "rapida" | "rápida" | "quick" => Ok(Self::Fast),
            "low" | "baixa" => Ok(Self::Low),
            "normal" | "balanced" => Ok(Self::Normal),
            "high" | "alta" => Ok(Self::High),
            "maximum" | "maxima" | "máxima" | "max" => Ok(Self::Maximum),
            _ => Err(format!("nível de compactação desconhecido: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEntry {
    pub path: PathBuf,
    pub kind: InputKind,
    pub size_bytes: Option<u64>,
}

impl InputEntry {
    pub fn new(path: PathBuf, kind: InputKind, size_bytes: Option<u64>) -> Self {
        Self {
            path,
            kind,
            size_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileClassification {
    Text,
    SourceCode,
    StructuredData,
    Document,
    CompressibleImage,
    CompressedImage,
    UncompressedAudio,
    CompressedAudio,
    Video,
    Archive,
    Executable,
    Database,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Classification {
    pub kind: FileClassification,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationPhase {
    Analyzing,
    Preparing,
    Compressing,
    Finalizing,
    Validating,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    Pending,
    Running(OperationPhase),
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(u64);

impl OperationId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "op-{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationState {
    NotInstalled,
    Installed,
    PartiallyInstalled,
    Broken,
    Unknown,
    RepairRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceProfile {
    pub available_memory_bytes: u64,
    pub cpu_count: usize,
    pub max_workers: usize,
    pub lightweight_mode: bool,
}

impl Default for ResourceProfile {
    fn default() -> Self {
        let cpu_count = std::thread::available_parallelism().map_or(1, usize::from);
        Self {
            available_memory_bytes: 0,
            cpu_count,
            max_workers: cpu_count.min(4),
            lightweight_mode: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionStrategy {
    pub algorithm_id: String,
    pub level: CompressionLevel,
    pub rationale: String,
    pub estimated_gain_percent: u8,
    pub parallel: bool,
}
