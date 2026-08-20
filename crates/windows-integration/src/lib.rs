pub mod context_menu;
pub mod launcher_protocol;
pub mod registry;

use compactador_core::models::InstallationState;
use registry::{InstallationDefinition, InstallationReport};
use std::path::Path;

pub fn expected_definition(executable_path: impl AsRef<Path>) -> InstallationDefinition {
    let executable_path = executable_path.as_ref().to_path_buf();
    let command = format!("\"{}\" --compress", executable_path.display());
    let entries = context_menu::default_commands()
        .into_iter()
        .map(|item| registry::RegistryEntry {
            hive: registry::RegistryHive::CurrentUser,
            key: format!("Software\\Classes\\*\\shell\\{}\\command", item.verb),
            value_name: None,
            value: command.clone(),
        })
        .collect();
    InstallationDefinition {
        application_id: context_menu::APPLICATION_ID.to_owned(),
        executable_path,
        entries,
    }
}

pub fn not_supported_report(message: impl Into<String>) -> InstallationReport {
    InstallationReport {
        state: InstallationState::Unknown,
        changed_entries: 0,
        verified: false,
        messages: vec![message.into()],
    }
}
