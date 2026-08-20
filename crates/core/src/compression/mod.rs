use crate::error::CoreResult;
use crate::models::{CompressionLevel, FileClassification};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedProfile {
    VeryFast,
    Fast,
    Balanced,
    Slow,
    VerySlow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub speed: SpeedProfile,
    pub memory_bytes_hint: u64,
    pub compression_potential: u8,
    pub streaming: bool,
    pub parallel: bool,
    pub minimum_useful_size: u64,
    pub supports_levels: &'static [CompressionLevel],
    pub suitable_for: &'static [FileClassification],
}

pub trait CompressionAlgorithm: Send + Sync {
    fn descriptor(&self) -> &'static AlgorithmDescriptor;
    fn compress(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        level: CompressionLevel,
    ) -> CoreResult<u64>;
}
