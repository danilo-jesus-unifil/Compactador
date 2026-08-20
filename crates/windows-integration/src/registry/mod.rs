use compactador_core::models::InstallationState;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryHive {
    CurrentUser,
    LocalMachine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub hive: RegistryHive,
    pub key: String,
    pub value_name: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationDefinition {
    pub application_id: String,
    pub executable_path: PathBuf,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationReport {
    pub state: InstallationState,
    pub changed_entries: usize,
    pub verified: bool,
    pub messages: Vec<String>,
}

pub trait RegistryBackend {
    type Error: std::error::Error + Send + Sync + 'static;

    fn read(
        &self,
        hive: RegistryHive,
        key: &str,
        value_name: Option<&str>,
    ) -> Result<Option<String>, Self::Error>;
    fn write(&self, entry: &RegistryEntry) -> Result<(), Self::Error>;
    fn delete(&self, entry: &RegistryEntry) -> Result<(), Self::Error>;
}
