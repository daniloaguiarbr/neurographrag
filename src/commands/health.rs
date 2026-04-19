use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use serde::Serialize;

#[derive(clap::Args)]
pub struct HealthArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
    /// Flag explícita de saída JSON. Aceita como no-op pois o output já é JSON por default.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
struct HealthCounts {
    memories: i64,
    entities: i64,
    relationships: i64,
    vec_memories: i64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    integrity: String,
    counts: HealthCounts,
    db_path: String,
    /// Versão do schema aplicado (top-level para contrato documentado em AGENT_PROTOCOL.md).
    schema_version: String,
    /// Lista de entidades referenciadas por memórias mas ausentes na tabela de entidades.
    /// Vazio em DB saudável. Conforme contrato documentado em AGENT_PROTOCOL.md.
    missing_entities: Vec<String>,
}

pub fn run(args: HealthArgs) -> Result<(), AppError> {
    let _ = args.json; // --json é no-op pois output já é JSON por default
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    let conn = open_ro(&paths.db)?;

    let integrity: String = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;

    let memories_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    let entities_count: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))?;
    let relationships_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM relationships", [], |r| r.get(0))?;
    let vec_memories_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM vec_memories", [], |r| r.get(0))?;

    let status = if integrity == "ok" { "ok" } else { "degraded" };

    let schema_version: String = conn
        .query_row(
            "SELECT value FROM schema_meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "unknown".to_string());

    // Detecta entidades órfãs referenciadas por memórias mas ausentes na tabela entities.
    let mut missing_entities: Vec<String> = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT me.entity_id
         FROM memory_entities me
         LEFT JOIN entities e ON e.id = me.entity_id
         WHERE e.id IS NULL",
    )?;
    let orphans: Vec<i64> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in orphans {
        missing_entities.push(format!("entity_id={id}"));
    }

    output::emit_json(&HealthResponse {
        status: status.to_string(),
        integrity,
        counts: HealthCounts {
            memories: memories_count,
            entities: entities_count,
            relationships: relationships_count,
            vec_memories: vec_memories_count,
        },
        db_path: paths.db.display().to_string(),
        schema_version,
        missing_entities,
    })?;

    Ok(())
}
