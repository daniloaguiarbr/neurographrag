use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use serde::Serialize;

#[derive(clap::Args)]
pub struct HealthArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
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
}

pub fn run(args: HealthArgs) -> Result<(), AppError> {
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
    })?;

    Ok(())
}
