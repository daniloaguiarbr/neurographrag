use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use crate::storage::{memories, versions};
use serde::Serialize;

#[derive(clap::Args)]
pub struct RenameArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub new_name: String,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct RenameResponse {
    memory_id: i64,
    name: String,
    version: i64,
}

pub fn run(args: RenameArgs) -> Result<(), AppError> {
    use crate::constants::*;

    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;

    if args.new_name.is_empty() || args.new_name.len() > MAX_MEMORY_NAME_LEN {
        return Err(AppError::Validation(format!(
            "new-name must be 1-{MAX_MEMORY_NAME_LEN} chars"
        )));
    }

    {
        let slug_re = regex::Regex::new(crate::constants::SLUG_REGEX)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("regex: {e}")))?;
        if !slug_re.is_match(&args.new_name) {
            return Err(AppError::Validation(format!(
                "new-name must be kebab-case slug (lowercase letters, digits, hyphens): '{}'",
                args.new_name
            )));
        }
    }

    let paths = AppPaths::resolve(args.db.as_deref())?;
    let mut conn = open_rw(&paths.db)?;

    let (memory_id, _, _) =
        memories::find_by_name(&conn, &namespace, &args.name)?.ok_or_else(|| {
            AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            ))
        })?;

    let row = memories::read_by_name(&conn, &namespace, &args.name)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("memory not found before rename")))?;

    let memory_type = row.memory_type.clone();
    let description = row.description.clone();
    let body = row.body.clone();
    let metadata = row.metadata.clone();

    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

    tx.execute(
        "UPDATE memories SET name=?2 WHERE id=?1 AND deleted_at IS NULL",
        rusqlite::params![memory_id, args.new_name],
    )?;

    let next_v = versions::next_version(&tx, memory_id)?;

    versions::insert_version(
        &tx,
        memory_id,
        next_v,
        &args.new_name,
        &memory_type,
        &description,
        &body,
        &metadata,
        None,
        "rename",
    )?;

    tx.commit()?;

    output::emit_json(&RenameResponse {
        memory_id,
        name: args.new_name,
        version: next_v,
    })?;

    Ok(())
}
