use compactador_core::models::InstallationState;

fn print_help() {
    println!("Compactador Launcher/Manager");
    println!("Uso: compactador-launcher <install|verify|repair|remove>");
}

fn main() {
    let command = std::env::args().nth(1);
    match command.as_deref() {
        Some("install" | "verify" | "repair" | "remove") => {
            println!(
                "Operação '{}' será implementada na categoria de integração Windows.",
                command.unwrap_or_default()
            );
            println!("Estado atual: {:?}", InstallationState::Unknown);
        }
        _ => print_help(),
    }
}
