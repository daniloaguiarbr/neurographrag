use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct PurgeArgs {
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, default_value = "0")]
    pub older_than_seconds: i64,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct PurgeResponse {
    purged_count: usize,
    purged_names: Vec<String>,
    namespace: String,
    cutoff_epoch: i64,
    warnings: Vec<String>,
}

pub fn run(args: PurgeArgs) -> Result<(), AppError> {
    if args.older_than_seconds < 0 {
        return Err(AppError::Validation(
            "older-than-seconds must be >= 0".to_string(),
        ));
    }

    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;

    if !paths.db.exists() {
        return Err(AppError::NotFound(format!(
            "database not found at {}. Run 'neurographrag init' first.",
            paths.db.display()
        )));
    }

    let mut conn = open_rw(&paths.db)?;
    let cutoff_epoch = current_epoch()? - args.older_than_seconds;
    let candidates = select_candidates(&conn, &namespace, args.name.as_deref(), cutoff_epoch)?;

    if candidates.is_empty() && args.name.is_some() {
        return Err(AppError::NotFound(format!(
            "soft-deleted memory '{}' not found in namespace '{}'",
            args.name.unwrap_or_default(),
            namespace
        )));
    }

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let mut purged_names = Vec::with_capacity(candidates.len());
    let mut warnings = Vec::new();

    for (memory_id, name) in &candidates {
        if let Err(err) = tx.execute(
            "DELETE FROM vec_chunks WHERE memory_id = ?1",
            rusqlite::params![memory_id],
        ) {
            warnings.push(format!(
                "vec_chunks cleanup failed for memory_id {memory_id}: {err}"
            ));
        }
        if let Err(err) = tx.execute(
            "DELETE FROM vec_memories WHERE memory_id = ?1",
            rusqlite::params![memory_id],
        ) {
            warnings.push(format!(
                "vec_memories cleanup failed for memory_id {memory_id}: {err}"
            ));
        }
        tx.execute(
            "DELETE FROM memories WHERE id = ?1 AND namespace = ?2 AND deleted_at IS NOT NULL",
            rusqlite::params![memory_id, namespace],
        )?;
        purged_names.push(name.clone());
    }

    tx.commit()?;

    output::emit_json(&PurgeResponse {
        purged_count: purged_names.len(),
        purged_names,
        namespace,
        cutoff_epoch,
        warnings,
    })?;

    Ok(())
}

fn current_epoch() -> Result<i64, AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| AppError::Internal(anyhow::anyhow!("system clock error: {err}")))?;
    Ok(now.as_secs() as i64)
}

fn select_candidates(
    conn: &rusqlite::Connection,
    namespace: &str,
    name: Option<&str>,
    cutoff_epoch: i64,
) -> Result<Vec<(i64, String)>, AppError> {
    let query = if name.is_some() {
        "SELECT id, name FROM memories
         WHERE namespace = ?1 AND name = ?2 AND deleted_at IS NOT NULL AND deleted_at <= ?3
         ORDER BY deleted_at ASC"
    } else {
        "SELECT id, name FROM memories
         WHERE namespace = ?1 AND deleted_at IS NOT NULL AND deleted_at <= ?2
         ORDER BY deleted_at ASC"
    };

    let mut stmt = conn.prepare(query)?;
    let rows = if let Some(name) = name {
        stmt.query_map(rusqlite::params![namespace, name, cutoff_epoch], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(rusqlite::params![namespace, cutoff_epoch], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    };
    Ok(rows)
}
