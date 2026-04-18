use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use crate::storage::memories;
use rusqlite::params;
use serde::Serialize;

#[derive(clap::Args)]
pub struct HistoryArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct HistoryVersion {
    version: i64,
    name: String,
    #[serde(rename = "type")]
    memory_type: String,
    description: String,
    body: String,
    metadata: String,
    change_reason: String,
    changed_by: Option<String>,
    created_at: i64,
}

#[derive(Serialize)]
struct HistoryResponse {
    name: String,
    namespace: String,
    versions: Vec<HistoryVersion>,
}

pub fn run(args: HistoryArgs) -> Result<(), AppError> {
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;
    let conn = open_ro(&paths.db)?;

    let (memory_id, _, _) =
        memories::find_by_name(&conn, &namespace, &args.name)?.ok_or_else(|| {
            AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            ))
        })?;

    let mut stmt = conn.prepare(
        "SELECT version, name, type, description, body, metadata,
                change_reason, changed_by, created_at
         FROM memory_versions
         WHERE memory_id = ?1
         ORDER BY version ASC",
    )?;

    let versions = stmt
        .query_map(params![memory_id], |r| {
            Ok(HistoryVersion {
                version: r.get(0)?,
                name: r.get(1)?,
                memory_type: r.get(2)?,
                description: r.get(3)?,
                body: r.get(4)?,
                metadata: r.get(5)?,
                change_reason: r.get(6)?,
                changed_by: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    output::emit_json(&HistoryResponse {
        name: args.name,
        namespace,
        versions,
    })?;

    Ok(())
}
