use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::storage::connection::open_ro;
use crate::storage::memories;

#[derive(clap::Args)]
pub struct ReadArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "global")]
    pub namespace: Option<String>,
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
}

pub fn run(args: ReadArgs) -> Result<(), AppError> {
    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;
    let paths = AppPaths::resolve(args.db.as_deref())?;
    let conn = open_ro(&paths.db)?;

    match memories::read_by_name(&conn, &namespace, &args.name)? {
        Some(row) => output::emit_json(&row)?,
        None => {
            return Err(AppError::NotFound(format!(
                "memory '{}' not found in namespace '{}'",
                args.name, namespace
            )))
        }
    }

    Ok(())
}
