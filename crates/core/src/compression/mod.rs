use crate::error::CoreResult;
use crate::models::{CompressionLevel, FileClassification};
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::{self, Read, Write};

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

const ALL_LEVELS: &[CompressionLevel] = &CompressionLevel::ALL;
const COMPRESSIBLE_CATEGORIES: &[FileClassification] = &[
    FileClassification::Text,
    FileClassification::SourceCode,
    FileClassification::StructuredData,
    FileClassification::Document,
    FileClassification::CompressibleImage,
    FileClassification::UncompressedAudio,
    FileClassification::Database,
    FileClassification::Unknown,
];
const ALL_CATEGORIES: &[FileClassification] = &[
    FileClassification::Text,
    FileClassification::SourceCode,
    FileClassification::StructuredData,
    FileClassification::Document,
    FileClassification::CompressibleImage,
    FileClassification::CompressedImage,
    FileClassification::UncompressedAudio,
    FileClassification::CompressedAudio,
    FileClassification::Video,
    FileClassification::Archive,
    FileClassification::Executable,
    FileClassification::Database,
    FileClassification::Unknown,
];

static DEFLATE_DESCRIPTOR: AlgorithmDescriptor = AlgorithmDescriptor {
    id: "deflate",
    display_name: "Deflate",
    speed: SpeedProfile::Balanced,
    memory_bytes_hint: 8 * 1024 * 1024,
    compression_potential: 70,
    streaming: true,
    parallel: false,
    minimum_useful_size: 128,
    supports_levels: ALL_LEVELS,
    suitable_for: COMPRESSIBLE_CATEGORIES,
};

static STORE_DESCRIPTOR: AlgorithmDescriptor = AlgorithmDescriptor {
    id: "store",
    display_name: "Armazenamento direto",
    speed: SpeedProfile::VeryFast,
    memory_bytes_hint: 1024 * 1024,
    compression_potential: 0,
    streaming: true,
    parallel: false,
    minimum_useful_size: 0,
    supports_levels: ALL_LEVELS,
    suitable_for: ALL_CATEGORIES,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct DeflateAlgorithm;

impl CompressionAlgorithm for DeflateAlgorithm {
    fn descriptor(&self) -> &'static AlgorithmDescriptor {
        &DEFLATE_DESCRIPTOR
    }

    fn compress(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        level: CompressionLevel,
    ) -> CoreResult<u64> {
        let mut encoder =
            DeflateEncoder::new(output, Compression::new(u32::from(level.numeric_hint())));
        let written = io::copy(input, &mut encoder)?;
        encoder.finish()?.flush()?;
        Ok(written)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StoreAlgorithm;

impl CompressionAlgorithm for StoreAlgorithm {
    fn descriptor(&self) -> &'static AlgorithmDescriptor {
        &STORE_DESCRIPTOR
    }

    fn compress(
        &self,
        input: &mut dyn Read,
        output: &mut dyn Write,
        _level: CompressionLevel,
    ) -> CoreResult<u64> {
        Ok(io::copy(input, output)?)
    }
}

pub fn available_algorithms() -> Vec<Box<dyn CompressionAlgorithm>> {
    vec![Box::new(DeflateAlgorithm), Box::new(StoreAlgorithm)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_exposes_streaming_fallbacks() {
        let algorithms = available_algorithms();
        assert_eq!(algorithms.len(), 2);
        assert!(algorithms
            .iter()
            .all(|algorithm| algorithm.descriptor().streaming));
        assert_eq!(algorithms[1].descriptor().id, "store");
    }

    #[test]
    fn deflate_adapter_round_trips_through_a_decoder() {
        use flate2::read::DeflateDecoder;
        use std::io::Read;
        let input = b"dados repetidos ".repeat(128);
        let mut compressed = Vec::new();
        let written = DeflateAlgorithm
            .compress(
                &mut input.as_slice(),
                &mut compressed,
                CompressionLevel::Normal,
            )
            .expect("compress");
        assert_eq!(written, input.len() as u64);
        let mut decoder = DeflateDecoder::new(compressed.as_slice());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).expect("decode");
        assert_eq!(decoded, input);
    }
}
