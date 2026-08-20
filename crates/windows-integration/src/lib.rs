pub mod context_menu;
pub mod launcher_protocol;
pub mod manager;
pub mod registry;

use compactador_core::models::InstallationState;
use manager::InstallationManager;
use registry::{InstallationDefinition, InstallationReport, RegistryEntry, RegistryHive};
use std::path::Path;

const MENU_KEYS: &[&str] = &[
    "Software\\Classes\\*\\shell\\CompactadorInteligente",
    "Software\\Classes\\Directory\\shell\\CompactadorInteligente",
];
const COMMAND_STORE_ROOT: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CommandStore\\shell";

pub fn expected_definition(executable_path: impl AsRef<Path>) -> InstallationDefinition {
    let executable_path = executable_path.as_ref().to_path_buf();
    let commands = context_menu::default_commands();
    let subcommands = commands
        .iter()
        .map(|item| item.verb.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let mut entries = Vec::new();
    for menu_key in MENU_KEYS {
        entries.push(RegistryEntry {
            hive: RegistryHive::CurrentUser,
            key: (*menu_key).to_owned(),
            value_name: Some("MUIVerb".to_owned()),
            value: "Compactar".to_owned(),
        });
        entries.push(RegistryEntry {
            hive: RegistryHive::CurrentUser,
            key: (*menu_key).to_owned(),
            value_name: Some("SubCommands".to_owned()),
            value: subcommands.clone(),
        });
    }
    for item in commands {
        let command_key = format!("{COMMAND_STORE_ROOT}\\{}", item.verb);
        let command = format!(
            "\"{}\" --compress {} -- %*",
            executable_path.display(),
            item.level.numeric_hint()
        );
        entries.push(RegistryEntry {
            hive: RegistryHive::CurrentUser,
            key: command_key.clone(),
            value_name: Some("MUIVerb".to_owned()),
            value: item.label,
        });
        entries.push(RegistryEntry {
            hive: RegistryHive::CurrentUser,
            key: format!("{command_key}\\command"),
            value_name: None,
            value: command,
        });
    }
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

pub fn default_manager(executable_path: impl AsRef<Path>) -> InstallationReport {
    not_supported_report(format!(
        "backend do Registro disponível apenas no Windows; executável esperado: {}",
        executable_path.as_ref().display()
    ))
}

pub type InstallationManagerFor<B> = InstallationManager<B>;
