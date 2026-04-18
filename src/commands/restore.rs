use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use crate::storage::{memories, versions};
use rusqlite::params;
use serde::Serialize;

#[derive(clap::Args)]
pub struct RestoreArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub version: i64,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct RestoreResponse {
    memory_id: i64,
    name: String,
    version: i64,
    restored_from: i64,
}

pub fn run(args: RestoreArgs) -> Result<(), AppError> {
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;
    let mut conn = open_rw(&paths.db)?;

    let (memory_id, _, _) =
        memories::find_by_name(&conn, &namespace, &args.name)?.ok_or_else(|| {
            AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            ))
        })?;

    let version_row: (String, String, String, String, String) = {
        let mut stmt = conn.prepare(
            "SELECT name, type, description, body, metadata
             FROM memory_versions
             WHERE memory_id = ?1 AND version = ?2",
        )?;

        stmt.query_row(params![memory_id, args.version], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(|_| {
            AppError::NotFound(format!(
                "version {} not found for memory '{}'",
                args.version, args.name
            ))
        })?
    };

    let (old_name, old_type, old_description, old_body, old_metadata) = version_row;

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    tx.execute(
        "UPDATE memories SET name=?2, type=?3, description=?4, body=?5, body_hash=?6
         WHERE id=?1 AND deleted_at IS NULL",
        rusqlite::params![
            memory_id,
            old_name,
            old_type,
            old_description,
            old_body,
            blake3::hash(old_body.as_bytes()).to_hex().to_string()
        ],
    )?;

    let next_v = versions::next_version(&tx, memory_id)?;

    versions::insert_version(
        &tx,
        memory_id,
        next_v,
        &old_name,
        &old_type,
        &old_description,
        &old_body,
        &old_metadata,
        None,
        "restore",
    )?;

    tx.commit()?;

    output::emit_json(&RestoreResponse {
        memory_id,
        name: old_name,
        version: next_v,
        restored_from: args.version,
    })?;

    Ok(())
}
