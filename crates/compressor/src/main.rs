mod operation;

use compactador_core::error::CoreError;
use compactador_core::models::{OperationPhase, ResourceProfile};
use compactador_core::selection::SelectionRequest;
use operation::{run_operation, CancellationToken, ProgressEvent, ProgressReporter};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
struct ConsoleReporter;

impl ProgressReporter for ConsoleReporter {
    fn report(&self, event: ProgressEvent) {
        let progress = if event.total_bytes == 0 {
            0
        } else {
            event.completed_bytes.saturating_mul(100) / event.total_bytes
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

fn default_output(request: &SelectionRequest) -> Result<PathBuf, CoreError> {
    let first = request
        .inputs
        .first()
        .ok_or_else(|| CoreError::InvalidInput("seleção vazia".to_owned()))?;
    let parent = first.path.parent().unwrap_or_else(|| Path::new("."));
    let name = if request.inputs.len() == 1 {
        first
            .path
            .file_stem()
            .or_else(|| first.path.file_name())
            .ok_or_else(|| CoreError::InvalidInput("entrada sem nome".to_owned()))?
            .to_string_lossy()
            .into_owned()
    } else {
        "compactado".to_owned()
    };
    let base = parent.join(format!("{name}.zip"));
    if !base.exists() {
        return Ok(base);
    }
    for index in 1..=9999 {
        let candidate = parent.join(format!("{name} ({index}).zip"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(CoreError::InvalidInput(
        "não foi possível reservar um nome de saída disponível".to_owned(),
    ))
}

fn main() {
    let request = match SelectionRequest::parse(std::env::args_os().skip(1)) {
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
    let token = CancellationToken::default();
    let reporter = ConsoleReporter;
    match run_operation(
        &request,
        output,
        ResourceProfile::default(),
        &token,
        &reporter,
    ) {
        Ok(result) => println!(
            "Arquivo criado: {} ({} bytes)",
            result.output.display(),
            result.summary.total_compressed_bytes
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
