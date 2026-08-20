use compactador_core::models::{CompressionLevel, InputEntry};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherCommand {
    Install,
    Verify,
    Repair,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionRequest {
    pub level: CompressionLevel,
    pub inputs: Vec<InputEntry>,
    pub output: Option<PathBuf>,
}

impl CompressionRequest {
    pub fn new(level: CompressionLevel, inputs: Vec<InputEntry>) -> Self {
        Self {
            level,
            inputs,
            output: None,
        }
    }
}
