use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_rw;
use crate::storage::memories;
use serde::Serialize;

#[derive(clap::Args)]
pub struct ForgetArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct ForgetResponse {
    forgotten: bool,
    name: String,
    namespace: String,
}

pub fn run(args: ForgetArgs) -> Result<(), AppError> {
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;

    let conn = open_rw(&paths.db)?;

    let maybe_row = memories::read_by_name(&conn, &namespace, &args.name)?;
    let forgotten = memories::soft_delete(&conn, &namespace, &args.name)?;

    if !forgotten {
        return Err(AppError::NotFound(format!(
            "memory '{}' not found in namespace '{}'",
            args.name, namespace
        )));
    }

    if let Some(row) = maybe_row {
        // FTS5 external-content: manual `DELETE FROM fts_memories WHERE rowid=?`
        // corrompe o índice. A limpeza correta acontece via trigger `trg_fts_ad`
        // quando `purge` remove fisicamente a linha de `memories`. Entre soft-delete
        // e purge, as queries FTS filtram `m.deleted_at IS NULL` no JOIN.
        if let Err(e) = memories::delete_vec(&conn, row.id) {
            tracing::warn!(memory_id = row.id, error = %e, "vec cleanup failed — orphan vector left");
        }
    }

    output::emit_json(&ForgetResponse {
        forgotten: true,
        name: args.name,
        namespace,
    })?;

    Ok(())
}
