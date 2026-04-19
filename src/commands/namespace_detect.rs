use crate::errors::AppError;
use crate::namespace;
use crate::output;

#[derive(clap::Args)]
pub struct NamespaceDetectArgs {
    #[arg(long)]
    pub namespace: Option<String>,
    /// Flag explícita de saída JSON. Aceita como no-op pois o output já é JSON por default.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

pub fn run(args: NamespaceDetectArgs) -> Result<(), AppError> {
    let _ = args.json; // --json é no-op pois output já é JSON por default
    let resolution = namespace::detect_namespace(args.namespace.as_deref())?;
    output::emit_json(&resolution)?;
    Ok(())
}
