//! Contratos de dados para uma futura ponte entre shell/launcher e compressor.
//!
//! O executável atual usa argumentos CLI diretamente; estes tipos não são IPC
//! nem indicam que o Explorer já serializa uma `CompressionRequest`.

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
