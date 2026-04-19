use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct SyncSafeCopyArgs {
    /// Caminho do arquivo snapshot. Aceita alias `--output` para compatibilidade com doc bilíngue.
    #[arg(long, alias = "output")]
    pub dest: std::path::PathBuf,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct SyncSafeCopyResponse {
    source_db_path: String,
    dest_path: String,
    bytes_copied: u64,
    status: String,
}

pub fn run(args: SyncSafeCopyArgs) -> Result<(), AppError> {
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    if args.dest == paths.db {
        return Err(AppError::Validation(
            "destination path must differ from the source database path".to_string(),
        ));
    }

    if let Some(parent) = args.dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = open_rw(&paths.db)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(conn);

    let bytes_copied = std::fs::copy(&paths.db, &args.dest)?;

    // Aplica permissões 600 no snapshot em Unix para evitar vazamento em Dropbox/NFS compartilhado.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&args.dest)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&args.dest, perms)?;
    }

    output::emit_json(&SyncSafeCopyResponse {
        source_db_path: paths.db.display().to_string(),
        dest_path: args.dest.display().to_string(),
        bytes_copied,
        status: "ok".to_string(),
    })?;

    Ok(())
}
