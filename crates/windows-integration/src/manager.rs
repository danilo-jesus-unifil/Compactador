use crate::registry::{InstallationDefinition, InstallationReport, RegistryBackend, RegistryEntry};
use compactador_core::models::InstallationState;

#[derive(Debug)]
pub struct InstallationManager<B> {
    backend: B,
    definition: InstallationDefinition,
}

impl<B> InstallationManager<B>
where
    B: RegistryBackend,
{
    pub fn new(backend: B, definition: InstallationDefinition) -> Self {
        Self {
            backend,
            definition,
        }
    }

    pub fn definition(&self) -> &InstallationDefinition {
        &self.definition
    }

    pub fn inspect(&self) -> Result<InstallationState, B::Error> {
        let mut present = 0_usize;
        let mut mismatched = 0_usize;
        for entry in &self.definition.entries {
            match self
                .backend
                .read(entry.hive, &entry.key, entry.value_name.as_deref())?
            {
                Some(value) if value == entry.value => present += 1,
                Some(_) => mismatched += 1,
                None => {}
            }
        }
        let total = self.definition.entries.len();
        Ok(if total == 0 {
            InstallationState::Unknown
        } else if present == total {
            InstallationState::Installed
        } else if present == 0 && mismatched == 0 {
            InstallationState::NotInstalled
        } else if mismatched > 0 {
            InstallationState::RepairRequired
        } else {
            InstallationState::PartiallyInstalled
        })
    }

    pub fn install(&self) -> Result<InstallationReport, B::Error> {
        let before = self.inspect()?;
        if before == InstallationState::Installed {
            return Ok(self.report(
                InstallationState::Installed,
                0,
                true,
                "integração já instalada; nenhuma alteração necessária",
            ));
        }

        let mut changed = 0_usize;
        for entry in &self.definition.entries {
            self.backend.write(entry)?;
            changed += 1;
        }
        let after = self.inspect()?;
        let verified = after == InstallationState::Installed;
        Ok(self.report(
            if verified {
                InstallationState::Installed
            } else {
                InstallationState::Broken
            },
            changed,
            verified,
            if verified {
                "integração instalada e verificada"
            } else {
                "instalação aplicada, mas a verificação falhou"
            },
        ))
    }

    pub fn verify(&self) -> Result<InstallationReport, B::Error> {
        let state = self.inspect()?;
        Ok(self.report(
            state,
            0,
            state == InstallationState::Installed,
            "estado da integração verificado",
        ))
    }

    pub fn repair(&self) -> Result<InstallationReport, B::Error> {
        let before = self.inspect()?;
        let report = self.install()?;
        let mut report = InstallationReport {
            messages: report.messages,
            ..report
        };
        report
            .messages
            .insert(0, format!("reparo iniciado a partir do estado {before:?}"));
        Ok(report)
    }

    pub fn remove(&self) -> Result<InstallationReport, B::Error> {
        let before = self.inspect()?;
        let mut changed = 0_usize;
        let mut messages = vec![format!("remoção iniciada a partir do estado {before:?}")];
        for entry in self.definition.entries.iter().rev() {
            if self.entry_matches(entry)? {
                self.backend.delete(entry)?;
                changed += 1;
            }
        }

        let mut after = self.inspect()?;
        if after != InstallationState::NotInstalled {
            messages.push(
                "a verificação primária encontrou resíduos conhecidos; iniciando limpeza restrita"
                    .to_owned(),
            );
            for entry in self.definition.entries.iter().rev() {
                if self.entry_matches(entry)? {
                    self.backend.delete(entry)?;
                    changed += 1;
                }
            }
            after = self.inspect()?;
        }
        let verified = after == InstallationState::NotInstalled;
        messages.push(
            if verified {
                "remoção concluída e verificada"
            } else {
                "remoção incompleta; recursos restantes pertencem ao escopo conhecido"
            }
            .to_owned(),
        );
        Ok(InstallationReport {
            state: after,
            changed_entries: changed,
            verified,
            messages,
        })
    }

    fn entry_matches(&self, entry: &RegistryEntry) -> Result<bool, B::Error> {
        Ok(self
            .backend
            .read(entry.hive, &entry.key, entry.value_name.as_deref())?
            .as_deref()
            == Some(entry.value.as_str()))
    }

    fn report(
        &self,
        state: InstallationState,
        changed_entries: usize,
        verified: bool,
        message: &str,
    ) -> InstallationReport {
        InstallationReport {
            state,
            changed_entries,
            verified,
            messages: vec![message.to_owned()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expected_definition;
    use crate::registry::InMemoryRegistry;
    use std::path::Path;

    #[test]
    fn installation_is_idempotent_and_repairable() {
        let definition = expected_definition(Path::new(
            "C:\\Program Files\\Compactador\\compactador-compressor.exe",
        ));
        let manager = InstallationManager::new(InMemoryRegistry::default(), definition);
        assert_eq!(
            manager.inspect().ok(),
            Some(InstallationState::NotInstalled)
        );
        let first = manager.install().expect("installation should succeed");
        assert!(first.verified);
        let second = manager
            .install()
            .expect("second installation should succeed");
        assert_eq!(second.changed_entries, 0);
        assert!(manager.repair().expect("repair should succeed").verified);
    }

    #[test]
    fn removal_only_targets_declared_entries() {
        let definition = expected_definition(Path::new("C:\\Compactador\\compressor.exe"));
        let registry = InMemoryRegistry::with_entries(definition.entries.clone());
        let foreign = RegistryEntry {
            hive: crate::registry::RegistryHive::CurrentUser,
            key: definition.entries[0].key.clone(),
            value_name: Some("ThirdPartyValue".to_owned()),
            value: "preserve-me".to_owned(),
        };
        registry
            .write(&foreign)
            .expect("foreign value should write");
        let manager = InstallationManager::new(registry.clone(), definition);
        let report = manager.remove().expect("removal should succeed");
        assert!(report.verified);
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry
                .read(foreign.hive, &foreign.key, foreign.value_name.as_deref())
                .ok()
                .flatten(),
            Some(foreign.value)
        );
    }

    #[test]
    fn removal_does_not_delete_mismatched_declared_value() {
        let definition = expected_definition(Path::new("C:\\Compactador\\compressor.exe"));
        let registry = InMemoryRegistry::with_entries(definition.entries.clone());
        let mismatched = definition.entries[0].clone();
        registry
            .write(&RegistryEntry {
                value: "foreign-owner".to_owned(),
                ..mismatched.clone()
            })
            .expect("mismatched value should write");
        let manager = InstallationManager::new(registry.clone(), definition);
        let report = manager.remove().expect("removal should complete");
        assert!(!report.verified);
        assert_eq!(report.state, InstallationState::RepairRequired);
        assert_eq!(
            registry
                .read(
                    mismatched.hive,
                    &mismatched.key,
                    mismatched.value_name.as_deref()
                )
                .ok()
                .flatten(),
            Some("foreign-owner".to_owned())
        );
    }
}
