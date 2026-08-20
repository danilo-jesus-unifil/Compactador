use crate::error::CoreResult;
use crate::models::{CompressionLevel, CompressionStrategy, ResourceProfile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputProfile {
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub has_compressed_content: bool,
    pub dominant_category: Option<crate::models::FileClassification>,
}

pub trait StrategySelector {
    fn select(
        &self,
        profile: &InputProfile,
        level: CompressionLevel,
        resources: &ResourceProfile,
    ) -> CoreResult<CompressionStrategy>;
}
