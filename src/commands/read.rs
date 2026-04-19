use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use crate::storage::memories;
use serde::Serialize;

#[derive(clap::Args)]
pub struct ReadArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct ReadResponse {
    /// Campo canônico do storage. Preservado para compatibilidade com clientes v2.0.0.
    id: i64,
    /// Alias semântico de `id` para contrato documentado em SKILL.md e AGENT_PROTOCOL.md.
    memory_id: i64,
    namespace: String,
    name: String,
    /// Alias semântico de `memory_type` para contrato documentado.
    #[serde(rename = "type")]
    type_alias: String,
    memory_type: String,
    description: String,
    body: String,
    body_hash: String,
    session_id: Option<String>,
    source: String,
    metadata: String,
    /// Versão mais recente da memória, útil para controle otimista via `--expected-updated-at`.
    version: i64,
    created_at: i64,
    /// Timestamp RFC 3339 UTC paralelo a `created_at` para parsers ISO 8601.
    created_at_iso: String,
    updated_at: i64,
    /// Timestamp RFC 3339 UTC paralelo a `updated_at` para parsers ISO 8601.
    updated_at_iso: String,
    /// Tempo total de execução em milissegundos desde início do handler até serialização.
    elapsed_ms: u64,
}

fn epoch_to_iso(epoch: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(epoch, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

pub fn run(args: ReadArgs) -> Result<(), AppError> {
    let inicio = std::time::Instant::now();
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;
    let conn = open_ro(&paths.db)?;

    match memories::read_by_name(&conn, &namespace, &args.name)? {
        Some(row) => {
            // Resolver versão atual via tabela memory_versions (maior version para este memory_id).
            let version: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(version), 1) FROM memory_versions WHERE memory_id=?1",
                    rusqlite::params![row.id],
                    |r| r.get(0),
                )
                .unwrap_or(1);

            let response = ReadResponse {
                id: row.id,
                memory_id: row.id,
                namespace: row.namespace,
                name: row.name,
                type_alias: row.memory_type.clone(),
                memory_type: row.memory_type,
                description: row.description,
                body: row.body,
                body_hash: row.body_hash,
                session_id: row.session_id,
                source: row.source,
                metadata: row.metadata,
                version,
                created_at: row.created_at,
                created_at_iso: epoch_to_iso(row.created_at),
                updated_at: row.updated_at,
                updated_at_iso: epoch_to_iso(row.updated_at),
                elapsed_ms: inicio.elapsed().as_millis() as u64,
            };
            output::emit_json(&response)?;
        }
        None => {
            return Err(AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            )))
        }
    }

    Ok(())
}
