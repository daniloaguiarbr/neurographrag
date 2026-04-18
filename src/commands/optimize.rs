use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct OptimizeArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct OptimizeResponse {
    db_path: String,
    status: String,
}

pub fn run(args: OptimizeArgs) -> Result<(), AppError> {
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    let conn = open_rw(&paths.db)?;
    conn.execute_batch("PRAGMA optimize;")?;

    output::emit_json(&OptimizeResponse {
        db_path: paths.db.display().to_string(),
        status: "ok".to_string(),
    })?;

    Ok(())
}
