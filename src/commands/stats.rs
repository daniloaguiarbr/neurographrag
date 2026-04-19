use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use serde::Serialize;

#[derive(clap::Args)]
pub struct StatsArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
    /// Flag explícita de saída JSON. Aceita como no-op pois o output já é JSON por default.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
struct StatsResponse {
    memories: i64,
    /// Alias de `memories` para contrato documentado em SKILL.md e AGENT_PROTOCOL.md.
    memories_total: i64,
    entities: i64,
    /// Alias de `entities` para contrato documentado.
    entities_total: i64,
    relationships: i64,
    /// Alias de `relationships` para contrato documentado.
    relationships_total: i64,
    /// Alias semântico de `relationships` conforme contrato em AGENT_PROTOCOL.md.
    edges: i64,
    /// Total de chunks indexados (linha por chunk em `memory_chunks`).
    chunks_total: i64,
    /// Comprimento médio do campo body nas memórias ativas (não deletadas).
    avg_body_len: f64,
    namespaces: Vec<String>,
    db_size_bytes: u64,
    /// Alias semântico de `db_size_bytes` para contrato documentado.
    db_bytes: u64,
    schema_version: String,
}

pub fn run(args: StatsArgs) -> Result<(), AppError> {
    let _ = args.json; // --json é no-op pois output já é JSON por default
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    let conn = open_ro(&paths.db)?;

    let memories: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    let entities: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?;
    let relationships: i64 =
        conn.query_row("SELECT COUNT(*) FROM relationships", [], |r| r.get(0))?;

    let mut stmt = conn.prepare(
        "SELECT DISTINCT namespace FROM memories WHERE deleted_at IS NULL ORDER BY namespace",
    )?;
    let namespaces: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let schema_version: String = conn
        .query_row(
            "SELECT value FROM schema_meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "unknown".to_string());

    let db_size_bytes = std::fs::metadata(&paths.db).map(|m| m.len()).unwrap_or(0);

    let chunks_total: i64 = conn
        .query_row("SELECT COUNT(*) FROM memory_chunks", [], |r| r.get(0))
        .unwrap_or(0);

    let avg_body_len: f64 = conn
        .query_row(
            "SELECT COALESCE(AVG(LENGTH(body)), 0.0) FROM memories WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    output::emit_json(&StatsResponse {
        memories,
        memories_total: memories,
        entities,
        entities_total: entities,
        relationships,
        relationships_total: relationships,
        edges: relationships,
        chunks_total,
        avg_body_len,
        namespaces,
        db_size_bytes,
        db_bytes: db_size_bytes,
        schema_version,
    })?;

    Ok(())
}
