use compactador_core::analysis::analyze_selection;
use compactador_core::container::{
    compress_inputs_with_strategy_and_validation, ArchiveSummary, ContainerCompression,
    ContainerProgressCallbacks,
};
use compactador_core::error::{CoreError, CoreResult};
use compactador_core::models::{OperationId, OperationPhase, ResourceProfile};
use compactador_core::selection::{
    HeuristicStrategySelector, InputProfile, SelectionRequest, StrategySelector,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressEvent {
    pub operation_id: OperationId,
    pub phase: OperationPhase,
    pub completed_bytes: u64,
    pub total_bytes: u64,
    pub message: String,
}

pub trait ProgressReporter: Send + Sync {
    fn report(&self, event: ProgressEvent);
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct NullReporter;

#[cfg(test)]
impl ProgressReporter for NullReporter {
    fn report(&self, _event: ProgressEvent) {}
}

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Hook usado por uma UI, handler de sinal ou integração futura para cancelar cooperativamente.
    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub operation_id: OperationId,
    pub output: PathBuf,
    pub summary: ArchiveSummary,
    pub strategy: compactador_core::models::CompressionStrategy,
}

pub fn run_operation(
    request: &SelectionRequest,
    output: PathBuf,
    resources: ResourceProfile,
    token: &CancellationToken,
    reporter: &dyn ProgressReporter,
) -> CoreResult<OperationResult> {
    let operation_id = OperationId::new();
    let initial_total_bytes = request.inputs.iter().fold(0_u64, |total, input| {
        total.saturating_add(input.size_bytes.unwrap_or(0))
    });
    report(
        reporter,
        operation_id,
        OperationPhase::Analyzing,
        0,
        initial_total_bytes,
        "analisando seleção",
    );
    ensure_not_cancelled(token, reporter, operation_id, initial_total_bytes)?;
    let analysis = analyze_selection(&request.inputs).map_err(CoreError::from)?;
    let total_bytes = analysis.total_size_bytes;
    report(
        reporter,
        operation_id,
        OperationPhase::Analyzing,
        0,
        total_bytes,
        "análise concluída",
    );
    ensure_not_cancelled(token, reporter, operation_id, total_bytes)?;
    let profile = InputProfile {
        total_size_bytes: analysis.total_size_bytes,
        file_count: analysis.files,
        directory_count: analysis.directories,
        has_compressed_content: analysis.already_compressed,
        dominant_category: analysis.dominant_category,
    };
    let strategy = HeuristicStrategySelector.select(&profile, request.level, &resources)?;
    report(
        reporter,
        operation_id,
        OperationPhase::Preparing,
        0,
        total_bytes,
        format!(
            "estratégia: {}; {}",
            strategy.algorithm_id, strategy.rationale
        ),
    );
    ensure_not_cancelled(token, reporter, operation_id, total_bytes)?;
    report(
        reporter,
        operation_id,
        OperationPhase::Compressing,
        0,
        total_bytes,
        "compactando em streaming",
    );
    let compression = match strategy.algorithm_id.as_str() {
        "store" => ContainerCompression::Store,
        "deflate" => ContainerCompression::Deflate,
        algorithm => {
            return Err(CoreError::InvalidConfiguration(format!(
                "algoritmo selecionado não suportado pelo container: {algorithm}"
            )))
        }
    };
    let completed_bytes = Arc::new(AtomicU64::new(0));
    let progress_state = Arc::clone(&completed_bytes);
    let summary = match compress_inputs_with_strategy_and_validation(
        request
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect::<Vec<_>>(),
        &output,
        request.level,
        compression,
        &|| token.is_cancelled(),
        ContainerProgressCallbacks {
            on_progress: &|completed_bytes| {
                progress_state.store(completed_bytes, Ordering::Release);
                report(
                    reporter,
                    operation_id,
                    OperationPhase::Compressing,
                    completed_bytes,
                    total_bytes,
                    "compactando em streaming",
                );
            },
            on_validation_start: &|| {
                report(
                    reporter,
                    operation_id,
                    OperationPhase::Validating,
                    0,
                    total_bytes,
                    "validando integridade do container",
                );
            },
            on_finalizing_start: &|| {
                report(
                    reporter,
                    operation_id,
                    OperationPhase::Finalizing,
                    total_bytes,
                    total_bytes,
                    "finalizando arquivo",
                );
            },
        },
    ) {
        Ok(summary) => summary,
        Err(CoreError::Cancelled) => {
            report(
                reporter,
                operation_id,
                OperationPhase::Cancelled,
                completed_bytes.load(Ordering::Acquire),
                total_bytes,
                "operação cancelada; temporários foram descartados",
            );
            return Err(CoreError::Cancelled);
        }
        Err(error) => return Err(error),
    };
    report(
        reporter,
        operation_id,
        OperationPhase::Completed,
        total_bytes,
        total_bytes,
        "compactação concluída",
    );
    Ok(OperationResult {
        operation_id,
        output,
        summary,
        strategy,
    })
}

fn ensure_not_cancelled(
    token: &CancellationToken,
    reporter: &dyn ProgressReporter,
    operation_id: OperationId,
    total_bytes: u64,
) -> CoreResult<()> {
    if token.is_cancelled() {
        report(
            reporter,
            operation_id,
            OperationPhase::Cancelled,
            0,
            total_bytes,
            "operação cancelada antes do início da próxima fase",
        );
        Err(CoreError::Cancelled)
    } else {
        Ok(())
    }
}

fn report(
    reporter: &dyn ProgressReporter,
    operation_id: OperationId,
    phase: OperationPhase,
    completed_bytes: u64,
    total_bytes: u64,
    message: impl Into<String>,
) {
    reporter.report(ProgressEvent {
        operation_id,
        phase,
        completed_bytes,
        total_bytes,
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use compactador_core::models::CompressionLevel;
    use compactador_core::selection::SelectionRequest;
    use std::ffi::OsString;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Default)]
    struct RecordingReporter(Mutex<Vec<ProgressEvent>>);

    impl ProgressReporter for RecordingReporter {
        fn report(&self, event: ProgressEvent) {
            self.0.lock().expect("lock").push(event);
        }
    }

    struct CancellingReporter {
        token: CancellationToken,
        events: Mutex<Vec<ProgressEvent>>,
    }

    struct PhaseCancellingReporter {
        token: CancellationToken,
        phase: OperationPhase,
        events: Mutex<Vec<ProgressEvent>>,
    }

    impl ProgressReporter for PhaseCancellingReporter {
        fn report(&self, event: ProgressEvent) {
            self.events.lock().expect("lock").push(event.clone());
            if event.phase == self.phase {
                self.token.cancel();
            }
        }
    }

    impl ProgressReporter for CancellingReporter {
        fn report(&self, event: ProgressEvent) {
            self.events.lock().expect("lock").push(event.clone());
            if event.phase == OperationPhase::Compressing && event.completed_bytes > 0 {
                self.token.cancel();
            }
        }
    }

    #[test]
    fn reports_real_phases_and_creates_output() {
        let root = std::env::temp_dir().join(format!(
            "compactador-operation-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("entrada.txt");
        let output = root.join("saida.zip");
        fs::write(&input, b"dados repetidos dados repetidos").expect("write input");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--"),
            input.as_os_str().to_os_string(),
        ])
        .expect("request");
        let reporter = RecordingReporter::default();
        let result = run_operation(
            &request,
            output.clone(),
            ResourceProfile::default(),
            &CancellationToken::default(),
            &reporter,
        )
        .expect("operation");
        assert_eq!(result.strategy.level, CompressionLevel::Normal);
        assert!(output.exists());
        let events = reporter.0.lock().expect("lock").clone();
        let phases = events.iter().map(|event| event.phase).collect::<Vec<_>>();
        assert!(phases.contains(&OperationPhase::Analyzing));
        assert!(phases.contains(&OperationPhase::Validating));
        assert!(phases.contains(&OperationPhase::Completed));
        let last_compressing = phases
            .iter()
            .rposition(|phase| *phase == OperationPhase::Compressing)
            .expect("compressing phase");
        let validating = phases
            .iter()
            .position(|phase| *phase == OperationPhase::Validating)
            .expect("validating phase");
        let finalizing = phases
            .iter()
            .position(|phase| *phase == OperationPhase::Finalizing)
            .expect("finalizing phase");
        let completed = phases
            .iter()
            .position(|phase| *phase == OperationPhase::Completed)
            .expect("completed phase");
        assert!(last_compressing < validating);
        assert!(validating < finalizing);
        assert!(finalizing < completed);
        assert!(events
            .iter()
            .any(|event| event.phase == OperationPhase::Compressing && event.completed_bytes > 0));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn compresses_empty_directory() {
        let root = std::env::temp_dir().join(format!(
            "compactador-empty-dir-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("pasta vazia");
        let output = root.join("saida.zip");
        fs::create_dir(&input).expect("create empty directory");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--"),
            input.as_os_str().to_os_string(),
        ])
        .expect("request");
        let result = run_operation(
            &request,
            output.clone(),
            ResourceProfile::default(),
            &CancellationToken::default(),
            &NullReporter,
        )
        .expect("empty directory operation");
        assert!(output.exists());
        assert!(result
            .summary
            .entries
            .iter()
            .any(|entry| entry.is_directory));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cancellation_during_streaming_discards_partial_output() {
        let root = std::env::temp_dir().join(format!(
            "compactador-cancel-stream-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("entrada.bin");
        fs::write(&input, vec![b'x'; 256 * 1024]).expect("write input");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--"),
            input.as_os_str().to_os_string(),
        ])
        .expect("request");
        let output = root.join("saida.zip");
        let token = CancellationToken::default();
        let reporter = CancellingReporter {
            token: token.clone(),
            events: Mutex::new(Vec::new()),
        };
        let result = run_operation(
            &request,
            output.clone(),
            ResourceProfile::default(),
            &token,
            &reporter,
        );
        assert!(matches!(result, Err(CoreError::Cancelled)));
        assert!(!output.exists());
        let events = reporter.events.lock().expect("lock").clone();
        assert_eq!(
            events.last().map(|event| event.phase),
            Some(OperationPhase::Cancelled)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cancellation_after_validation_start_discards_output() {
        let root = std::env::temp_dir().join(format!(
            "compactador-cancel-validation-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("entrada.txt");
        let output = root.join("saida.zip");
        fs::write(&input, b"dados repetidos dados repetidos").expect("write input");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--"),
            input.as_os_str().to_os_string(),
        ])
        .expect("request");
        let token = CancellationToken::default();
        let reporter = PhaseCancellingReporter {
            token: token.clone(),
            phase: OperationPhase::Validating,
            events: Mutex::new(Vec::new()),
        };
        let result = run_operation(
            &request,
            output.clone(),
            ResourceProfile::default(),
            &token,
            &reporter,
        );
        assert!(matches!(result, Err(CoreError::Cancelled)));
        assert!(!output.exists());
        let events = reporter.events.lock().expect("lock").clone();
        assert_eq!(
            events.last().map(|event| event.phase),
            Some(OperationPhase::Cancelled)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cancellation_is_reported_before_work_starts() {
        let root = std::env::temp_dir().join(format!(
            "compactador-cancel-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("entrada.txt");
        fs::write(&input, b"dados").expect("write input");
        let request = SelectionRequest::parse([
            OsString::from("--compress"),
            OsString::from("normal"),
            OsString::from("--"),
            input.as_os_str().to_os_string(),
        ])
        .expect("request");
        let token = CancellationToken::default();
        let reporter = PhaseCancellingReporter {
            token: token.clone(),
            phase: OperationPhase::Analyzing,
            events: Mutex::new(Vec::new()),
        };
        let result = run_operation(
            &request,
            root.join("saida.zip"),
            ResourceProfile::default(),
            &token,
            &reporter,
        );
        assert!(matches!(result, Err(CoreError::Cancelled)));
        let events = reporter.events.lock().expect("lock").clone();
        assert_eq!(
            events.last().map(|event| event.phase),
            Some(OperationPhase::Cancelled)
        );
        let _ = fs::remove_dir_all(root);
    }
}
