use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use serde::Serialize;

#[derive(clap::Args)]
pub struct StatsArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct StatsResponse {
    memories: i64,
    entities: i64,
    relationships: i64,
    namespaces: Vec<String>,
    db_size_bytes: u64,
    schema_version: String,
}

pub fn run(args: StatsArgs) -> Result<(), AppError> {
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

    output::emit_json(&StatsResponse {
        memories,
        entities,
        relationships,
        namespaces,
        db_size_bytes,
        schema_version,
    })?;

    Ok(())
}
