use compactador_core::models::CompressionLevel;

pub const APPLICATION_ID: &str = "CompactadorInteligente";
pub const ROOT_VERB: &str = "compactador_compress";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuCommand {
    pub verb: String,
    pub label: String,
    pub level: CompressionLevel,
}

pub fn default_commands() -> Vec<ContextMenuCommand> {
    CompressionLevel::ALL
        .into_iter()
        .map(|level| ContextMenuCommand {
            verb: format!("{ROOT_VERB}_{}", level.numeric_hint()),
            label: level.display_name().to_owned(),
            level,
        })
        .collect()
}
