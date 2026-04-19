use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct VacuumArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct VacuumResponse {
    db_path: String,
    size_before_bytes: u64,
    size_after_bytes: u64,
    status: String,
    /// Tempo total de execução em milissegundos desde início do handler até serialização.
    elapsed_ms: u64,
}

pub fn run(args: VacuumArgs) -> Result<(), AppError> {
    let inicio = std::time::Instant::now();
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    let size_before_bytes = std::fs::metadata(&paths.db)
        .map(|meta| meta.len())
        .unwrap_or(0);
    let conn = open_rw(&paths.db)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    conn.execute_batch("VACUUM;")?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(conn);
    let size_after_bytes = std::fs::metadata(&paths.db)
        .map(|meta| meta.len())
        .unwrap_or(0);

    output::emit_json(&VacuumResponse {
        db_path: paths.db.display().to_string(),
        size_before_bytes,
        size_after_bytes,
        status: "ok".to_string(),
        elapsed_ms: inicio.elapsed().as_millis() as u64,
    })?;

    Ok(())
}
