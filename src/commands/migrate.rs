use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct MigrateArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
    /// Flag explícita de saída JSON. Aceita como no-op pois o output já é JSON por default.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
struct MigrateResponse {
    db_path: String,
    schema_version: String,
    status: String,
}

pub fn run(args: MigrateArgs) -> Result<(), AppError> {
    let _ = args.json; // --json é no-op pois output já é JSON por default
    let paths = AppPaths::resolve(args.db.as_deref())?;
    paths.ensure_dirs()?;

    let mut conn = open_rw(&paths.db)?;
    crate::migrations::runner()
        .run(&mut conn)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("migration failed: {e}")))?;

    let schema_version = latest_schema_version(&conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('schema_version', ?1)",
        rusqlite::params![schema_version],
    )?;

    output::emit_json(&MigrateResponse {
        db_path: paths.db.display().to_string(),
        schema_version,
        status: "ok".to_string(),
    })?;

    Ok(())
}

fn latest_schema_version(conn: &rusqlite::Connection) -> Result<String, AppError> {
    match conn.query_row(
        "SELECT version FROM refinery_schema_history ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(version) => Ok(version.to_string()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok("0".to_string()),
        Err(err) => Err(AppError::Database(err)),
    }
}
