pub mod analysis;
pub mod compression;
pub mod container;
pub mod error;
pub mod filesystem;
pub mod models;
pub mod security;
pub mod selection;

pub use error::{CoreError, CoreResult};
pub use models::{
    Classification, CompressionLevel, CompressionStrategy, Confidence, FileClassification,
    InputEntry, InputKind, InstallationState, OperationId, OperationPhase, OperationStatus,
    ResourceProfile,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parses_all_public_compression_levels() {
        for level in CompressionLevel::ALL {
            let parsed = CompressionLevel::from_str(level.display_name());
            assert!(parsed.is_ok(), "level should parse: {level}");
            assert_eq!(parsed.unwrap_or(CompressionLevel::Normal), level);
        }
    }

    #[test]
    fn operation_ids_are_monotonic_for_local_generation() {
        let first = OperationId::new().raw();
        let second = OperationId::new().raw();
        assert!(second > first);
    }
}
