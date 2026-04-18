use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use crate::storage::{memories, versions};
use serde::Serialize;
use std::io::Read as _;

#[derive(clap::Args)]
pub struct EditArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,
    #[arg(long)]
    pub body_stdin: bool,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub expected_updated_at: Option<i64>,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct EditResponse {
    memory_id: i64,
    name: String,
    action: String,
    version: i64,
}

pub fn run(args: EditArgs) -> Result<(), AppError> {
    use crate::constants::*;

    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;

    let paths = AppPaths::resolve(args.db.as_deref())?;
    let mut conn = open_rw(&paths.db)?;

    let (memory_id, current_updated_at, _current_version) =
        memories::find_by_name(&conn, &namespace, &args.name)?.ok_or_else(|| {
            AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            ))
        })?;

    if let Some(expected) = args.expected_updated_at {
        if expected != current_updated_at {
            return Err(AppError::Conflict(format!(
                "optimistic lock conflict: expected updated_at={expected}, but current is {current_updated_at}"
            )));
        }
    }

    let mut raw_body: Option<String> = None;
    if args.body.is_some() || args.body_file.is_some() || args.body_stdin {
        let b = if let Some(b) = args.body {
            b
        } else if let Some(path) = &args.body_file {
            std::fs::read_to_string(path).map_err(AppError::Io)?
        } else {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(AppError::Io)?;
            buf
        };
        if b.len() > MAX_MEMORY_BODY_LEN {
            return Err(AppError::LimitExceeded(format!(
                "body exceeds {MAX_MEMORY_BODY_LEN} chars"
            )));
        }
        raw_body = Some(b);
    }

    if let Some(ref desc) = args.description {
        if desc.len() > MAX_MEMORY_DESCRIPTION_LEN {
            return Err(AppError::Validation(format!(
                "description exceeds {MAX_MEMORY_DESCRIPTION_LEN} chars"
            )));
        }
    }

    let row = memories::read_by_name(&conn, &namespace, &args.name)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("memory row not found after check")))?;

    let new_body = raw_body.unwrap_or(row.body.clone());
    let new_description = args.description.unwrap_or(row.description.clone());
    let new_hash = blake3::hash(new_body.as_bytes()).to_hex().to_string();
    let memory_type = row.memory_type.clone();
    let metadata = row.metadata.clone();

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    let affected = if let Some(ts) = args.expected_updated_at {
        tx.execute(
            "UPDATE memories SET description=?2, body=?3, body_hash=?4
             WHERE id=?1 AND updated_at=?5 AND deleted_at IS NULL",
            rusqlite::params![memory_id, new_description, new_body, new_hash, ts],
        )?
    } else {
        tx.execute(
            "UPDATE memories SET description=?2, body=?3, body_hash=?4
             WHERE id=?1 AND deleted_at IS NULL",
            rusqlite::params![memory_id, new_description, new_body, new_hash],
        )?
    };

    if affected == 0 {
        return Err(AppError::Conflict(
            "optimistic lock conflict: memory was modified by another process".to_string(),
        ));
    }

    let next_v = versions::next_version(&tx, memory_id)?;

    versions::insert_version(
        &tx,
        memory_id,
        next_v,
        &args.name,
        &memory_type,
        &new_description,
        &new_body,
        &metadata,
        None,
        "edit",
    )?;

    tx.commit()?;

    output::emit_json(&EditResponse {
        memory_id,
        name: args.name,
        action: "updated".to_string(),
        version: next_v,
    })?;

    Ok(())
}
