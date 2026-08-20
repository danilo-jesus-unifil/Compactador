use compactador_core::models::InstallationState;
#[cfg(not(windows))]
use compactador_windows_integration::not_supported_report;

use std::path::PathBuf;

fn print_help() {
    println!("Compactador Launcher/Manager");
    println!("Uso: compactador-launcher <install|verify|repair|remove>");
    println!("O executável operacional deve estar ao lado deste launcher.");
}

fn operational_executable() -> Result<PathBuf, String> {
    let mut path = std::env::current_exe()
        .map_err(|error| format!("não foi possível descobrir o executável atual: {error}"))?;
    path.set_file_name(if cfg!(windows) {
        "compactador-compressor.exe"
    } else {
        "compactador-compressor"
    });
    Ok(path)
}

#[cfg(windows)]
fn execute(
    command: &str,
    executable: PathBuf,
) -> Result<compactador_windows_integration::registry::InstallationReport, String> {
    use compactador_windows_integration::registry::WindowsRegistry;
    let manager = compactador_windows_integration::manager::InstallationManager::new(
        WindowsRegistry,
        compactador_windows_integration::expected_definition(executable),
    );
    match command {
        "install" => manager.install().map_err(|error| error.to_string()),
        "verify" => manager.verify().map_err(|error| error.to_string()),
        "repair" => manager.repair().map_err(|error| error.to_string()),
        "remove" => manager.remove().map_err(|error| error.to_string()),
        _ => Err("comando desconhecido".to_owned()),
    }
}

#[cfg(not(windows))]
fn execute(
    _command: &str,
    executable: PathBuf,
) -> Result<compactador_windows_integration::registry::InstallationReport, String> {
    Ok(not_supported_report(format!(
        "a integração do Explorer requer Windows; executável planejado: {}",
        executable.display()
    )))
}

fn main() {
    let command = std::env::args().nth(1);
    let Some(command) = command else {
        print_help();
        return;
    };
    if !matches!(command.as_str(), "install" | "verify" | "repair" | "remove") {
        eprintln!("comando inválido: {command}");
        print_help();
        std::process::exit(2);
    }

    let executable = match operational_executable() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    match execute(&command, executable) {
        Ok(report) => {
            println!("Estado: {:?}", report.state);
            for message in report.messages {
                println!("{message}");
            }
            if !report.verified && report.state != InstallationState::Unknown {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("falha na operação: {error}");
            std::process::exit(1);
        }
    }
}
