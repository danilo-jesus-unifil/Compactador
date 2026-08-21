mod operation;

use compactador_core::container::extract_archive_with_cancel;
use compactador_core::error::CoreError;
use compactador_core::models::{OperationPhase, ResourceProfile};
use compactador_core::selection::SelectionRequest;
use operation::{run_operation, CancellationToken, ProgressEvent, ProgressReporter};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
struct ConsoleReporter;

impl ProgressReporter for ConsoleReporter {
    fn report(&self, event: ProgressEvent) {
        let progress = if event.total_bytes == 0 && event.phase == OperationPhase::Completed {
            100
        } else {
            event
                .completed_bytes
                .saturating_mul(100)
                .checked_div(event.total_bytes)
                .unwrap_or(0)
        };
        println!(
            "[{}] {:>3}% {}",
            phase_name(event.phase),
            progress.min(100),
            event.message
        );
    }
}

fn phase_name(phase: OperationPhase) -> &'static str {
    match phase {
        OperationPhase::Analyzing => "Analisando",
        OperationPhase::Preparing => "Preparando",
        OperationPhase::Compressing => "Compactando",
        OperationPhase::Finalizing => "Finalizando",
        OperationPhase::Validating => "Validando",
        OperationPhase::Completed => "Concluído",
        OperationPhase::Cancelled => "Cancelado",
    }
}

fn compression_stem(request: &SelectionRequest) -> Result<OsString, CoreError> {
    let first = request
        .inputs
        .first()
        .ok_or_else(|| CoreError::InvalidInput("seleção vazia".to_owned()))?;
    if request.inputs.len() == 1 {
        first
            .path
            .file_stem()
            .or_else(|| first.path.file_name())
            .map(|name| name.to_os_string())
            .ok_or_else(|| CoreError::InvalidInput("entrada sem nome".to_owned()))
    } else {
        Ok(OsString::from("compactado"))
    }
}

fn default_output(request: &SelectionRequest) -> Result<PathBuf, CoreError> {
    let first = request
        .inputs
        .first()
        .ok_or_else(|| CoreError::InvalidInput("seleção vazia".to_owned()))?;
    let parent = first.path.parent().unwrap_or_else(|| Path::new("."));
    let name = compression_stem(request)?;
    let mut base_name = name;
    base_name.push(".zip");
    let base = parent.join(base_name);
    if !base.exists() {
        return Ok(base);
    }
    for index in 1..=9999 {
        let mut candidate_name = compression_stem(request)?;
        candidate_name.push(format!(" ({index}).zip"));
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(CoreError::InvalidInput(
        "não foi possível reservar um nome de saída disponível".to_owned(),
    ))
}

fn extraction_stem(archive: &Path) -> Result<OsString, CoreError> {
    archive
        .file_stem()
        .or_else(|| archive.file_name())
        .map(|name| name.to_os_string())
        .ok_or_else(|| CoreError::InvalidInput("arquivo ZIP sem nome".to_owned()))
}

fn default_extraction_output(archive: &Path) -> Result<PathBuf, CoreError> {
    let parent = archive.parent().unwrap_or_else(|| Path::new("."));
    let mut base_name = extraction_stem(archive)?;
    base_name.push("-extraido");
    let base = parent.join(base_name);
    if !base.exists() {
        return Ok(base);
    }
    for index in 1..=9999 {
        let mut candidate_name = extraction_stem(archive)?;
        candidate_name.push(format!("-extraido ({index})"));
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(CoreError::InvalidInput(
        "não foi possível reservar um destino de extração disponível".to_owned(),
    ))
}

fn parse_decompression(
    arguments: &[std::ffi::OsString],
) -> Result<Option<(PathBuf, PathBuf)>, CoreError> {
    if arguments.first().and_then(|value| value.to_str()) != Some("--decompress") {
        return Ok(None);
    }
    let mut output = None;
    let mut input = None;
    let mut after_separator = false;
    let mut index = 1;
    while index < arguments.len() {
        let argument = &arguments[index];
        if !after_separator && argument == "--" {
            after_separator = true;
        } else if !after_separator && argument == "--output" {
            index += 1;
            let value = arguments.get(index).ok_or_else(|| {
                CoreError::InvalidInput("--output exige um destino de extração".to_owned())
            })?;
            output = Some(PathBuf::from(value));
        } else if !after_separator && argument.to_string_lossy().starts_with('-') {
            return Err(CoreError::InvalidInput(format!(
                "opção desconhecida: {}",
                argument.to_string_lossy()
            )));
        } else if input.replace(PathBuf::from(argument)).is_some() {
            return Err(CoreError::InvalidInput(
                "a descompactação aceita exatamente um arquivo ZIP".to_owned(),
            ));
        }
        index += 1;
    }
    let input = input.ok_or_else(|| {
        CoreError::InvalidInput("nenhum arquivo ZIP foi fornecido para descompactação".to_owned())
    })?;
    let output = match output {
        Some(output) => output,
        None => default_extraction_output(&input)?,
    };
    Ok(Some((input, output)))
}

fn print_help() {
    println!("Compactador Compressor");
    println!("Uso: compactador-compressor --compress <fast|low|normal|high|maximum> [--output caminho] -- caminho...");
    println!("     compactador-compressor --decompress [--output diretório] -- arquivo.zip");
    println!("Aceita arquivos e diretórios; a saída padrão é criada ao lado da primeira entrada.");
}

fn main() {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if matches!(
        arguments.first().and_then(|value| value.to_str()),
        Some("-h") | Some("--help")
    ) {
        print_help();
        return;
    }
    let token = CancellationToken::default();
    let signal_token = token.clone();
    if let Err(error) = ctrlc::set_handler(move || signal_token.cancel()) {
        eprintln!("falha ao instalar handler de Ctrl+C: {error}");
        std::process::exit(1);
    }
    match parse_decompression(&arguments) {
        Ok(Some((archive, destination))) => {
            let destination_display = destination.display().to_string();
            match extract_archive_with_cancel(archive, destination, &|| token.is_cancelled()) {
                Ok(summary) => {
                    println!(
                        "Descompactação concluída em {} ({} entradas; {} bytes)",
                        destination_display,
                        summary.entries.len(),
                        summary.total_original_bytes
                    );
                    return;
                }
                Err(CoreError::Cancelled) => {
                    eprintln!("operação cancelada; temporários foram descartados");
                    std::process::exit(130);
                }
                Err(error) => {
                    eprintln!("falha na descompactação: {error}");
                    std::process::exit(1);
                }
            }
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("falha ao receber descompactação: {error}");
            eprintln!(
                "Uso: compactador-compressor --decompress [--output diretório] -- arquivo.zip"
            );
            std::process::exit(2);
        }
    }
    let request = match SelectionRequest::parse(arguments) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("falha ao receber seleção: {error}");
            eprintln!("Uso: compactador-compressor --compress <fast|low|normal|high|maximum> [--output caminho] -- caminho...");
            std::process::exit(2);
        }
    };
    let output = match request
        .output
        .clone()
        .map(Ok)
        .unwrap_or_else(|| default_output(&request))
    {
        Ok(output) => output,
        Err(error) => {
            eprintln!("falha ao escolher saída: {error}");
            std::process::exit(1);
        }
    };
    let reporter = ConsoleReporter;
    match run_operation(
        &request,
        output,
        ResourceProfile::default(),
        &token,
        &reporter,
    ) {
        Ok(result) => println!(
            "Operação {} concluída com {} ({} bytes; estratégia {})",
            result.operation_id,
            result.output.display(),
            result.summary.total_compressed_bytes,
            result.strategy.algorithm_id
        ),
        Err(CoreError::Cancelled) => {
            eprintln!("operação cancelada; temporários foram descartados");
            std::process::exit(130);
        }
        Err(error) => {
            eprintln!("falha na compactação: {error}");
            std::process::exit(1);
        }
    }
}
