use crate::cli::MemoryType;
use crate::errors::AppError;
use crate::output::{self, OutputFormat};
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use crate::storage::memories;
use serde::Serialize;

#[derive(clap::Args)]
pub struct ListArgs {
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,
    #[arg(long, default_value = "50")]
    pub limit: usize,
    #[arg(long, default_value = "0")]
    pub offset: usize,
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

#[derive(Serialize)]
struct ListItem {
    id: i64,
    name: String,
    namespace: String,
    #[serde(rename = "type")]
    memory_type: String,
    description: String,
    snippet: String,
    updated_at: i64,
}

pub fn run(args: ListArgs) -> Result<(), AppError> {
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;
    let conn = open_ro(&paths.db)?;

    let memory_type_str = args.r#type.map(|t| t.as_str());
    let rows = memories::list(&conn, &namespace, memory_type_str, args.limit, args.offset)?;

    let items: Vec<ListItem> = rows
        .into_iter()
        .map(|r| {
            let snippet: String = r.body.chars().take(200).collect();
            ListItem {
                id: r.id,
                name: r.name,
                namespace: r.namespace,
                memory_type: r.memory_type,
                description: r.description,
                snippet,
                updated_at: r.updated_at,
            }
        })
        .collect();

    output::emit_json(&items)?;
    Ok(())
}
