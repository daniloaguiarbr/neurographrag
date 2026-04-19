use crate::errors::AppError;
use crate::namespace;
use crate::output;
use serde::Serialize;

#[derive(clap::Args)]
pub struct NamespaceDetectArgs {
    #[arg(long)]
    pub namespace: Option<String>,
    /// Flag explícita de saída JSON. Aceita como no-op pois o output já é JSON por default.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
struct NamespaceDetectResponse {
    namespace: String,
    source: namespace::NamespaceSource,
    cwd: String,
    project_config_path: String,
    projects_mapping_path: String,
    /// Tempo total de execução em milissegundos desde início do handler até serialização.
    elapsed_ms: u64,
}

pub fn run(args: NamespaceDetectArgs) -> Result<(), AppError> {
    let inicio = std::time::Instant::now();
    let _ = args.json; // --json é no-op pois output já é JSON por default
    let resolution = namespace::detect_namespace(args.namespace.as_deref())?;
    output::emit_json(&NamespaceDetectResponse {
        namespace: resolution.namespace,
        source: resolution.source,
        cwd: resolution.cwd,
        project_config_path: resolution.project_config_path,
        projects_mapping_path: resolution.projects_mapping_path,
        elapsed_ms: inicio.elapsed().as_millis() as u64,
    })?;
    Ok(())
}
