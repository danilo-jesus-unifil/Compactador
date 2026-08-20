use compactador_core::models::InstallationState;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryHive {
    CurrentUser,
    LocalMachine,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryError {
    pub message: String,
}

impl RegistryError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RegistryError {}

pub trait RegistryBackend: Send + Sync {
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

type RegistryKey = (RegistryHive, String, Option<String>);
type RegistryValues = HashMap<RegistryKey, String>;
type RegistryStore = Arc<Mutex<RegistryValues>>;

#[derive(Debug, Clone, Default)]
pub struct InMemoryRegistry {
    values: RegistryStore,
}

impl InMemoryRegistry {
    pub fn with_entries(entries: impl IntoIterator<Item = RegistryEntry>) -> Self {
        let registry = Self::default();
        for entry in entries {
            let _ = registry.write(&entry);
        }
        registry
    }

    pub fn len(&self) -> usize {
        self.values.lock().map_or(0, |values| values.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl RegistryBackend for InMemoryRegistry {
    type Error = RegistryError;

    fn read(
        &self,
        hive: RegistryHive,
        key: &str,
        value_name: Option<&str>,
    ) -> Result<Option<String>, Self::Error> {
        let values = self
            .values
            .lock()
            .map_err(|_| RegistryError::new("mutex do registro de teste envenenado"))?;
        Ok(values
            .get(&(hive, key.to_owned(), value_name.map(str::to_owned)))
            .cloned())
    }

    fn write(&self, entry: &RegistryEntry) -> Result<(), Self::Error> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| RegistryError::new("mutex do registro de teste envenenado"))?;
        values.insert(
            (entry.hive, entry.key.clone(), entry.value_name.clone()),
            entry.value.clone(),
        );
        Ok(())
    }

    fn delete(&self, entry: &RegistryEntry) -> Result<(), Self::Error> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| RegistryError::new("mutex do registro de teste envenenado"))?;
        values.remove(&(entry.hive, entry.key.clone(), entry.value_name.clone()));
        Ok(())
    }
}

#[cfg(windows)]
mod windows_backend {
    use super::*;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    #[derive(Debug, Clone, Default)]
    pub struct WindowsRegistry;

    impl WindowsRegistry {
        fn root(hive: RegistryHive) -> RegKey {
            match hive {
                RegistryHive::CurrentUser => RegKey::predef(HKEY_CURRENT_USER),
                RegistryHive::LocalMachine => RegKey::predef(HKEY_LOCAL_MACHINE),
            }
        }
    }

    impl RegistryBackend for WindowsRegistry {
        type Error = std::io::Error;

        fn read(
            &self,
            hive: RegistryHive,
            key: &str,
            value_name: Option<&str>,
        ) -> Result<Option<String>, Self::Error> {
            let root = Self::root(hive);
            let subkey = match root.open_subkey(key) {
                Ok(subkey) => subkey,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error),
            };
            match subkey.get_value::<String, _>(value_name.unwrap_or("")) {
                Ok(value) => Ok(Some(value)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error),
            }
        }

        fn write(&self, entry: &RegistryEntry) -> Result<(), Self::Error> {
            let root = Self::root(entry.hive);
            let (key, _) = root.create_subkey(&entry.key)?;
            key.set_value(entry.value_name.as_deref().unwrap_or(""), &entry.value)
        }

        fn delete(&self, entry: &RegistryEntry) -> Result<(), Self::Error> {
            let root = Self::root(entry.hive);
            let key = match root.open_subkey_with_flags(&entry.key, winreg::enums::KEY_WRITE) {
                Ok(key) => key,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error),
            };
            if let Some(value_name) = entry.value_name.as_deref() {
                key.delete_value(value_name)
            } else {
                drop(key);
                root.delete_subkey_all(&entry.key)
            }
        }
    }
}

#[cfg(windows)]
pub use windows_backend::WindowsRegistry;
