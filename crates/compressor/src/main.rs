use compactador_core::models::CompressionLevel;

fn print_help() {
    println!("Compactador operacional");
    println!("Uso: compactador-compressor --compress <fast|low|normal|high|maximum> <caminho>...");
}

fn main() {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("--compress"), Some(level)) => match level.parse::<CompressionLevel>() {
            Ok(level) => println!("Solicitação recebida: nível {level}. O motor será implementado nas categorias seguintes."),
            Err(error) => eprintln!("{error}"),
        },
        _ => print_help(),
    }
}
