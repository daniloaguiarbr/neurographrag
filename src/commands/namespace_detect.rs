use crate::errors::AppError;
use crate::namespace;
use crate::output;

#[derive(clap::Args)]
pub struct NamespaceDetectArgs {
    #[arg(long)]
    pub namespace: Option<String>,
}

pub fn run(args: NamespaceDetectArgs) -> Result<(), AppError> {
    let resolution = namespace::detect_namespace(args.namespace.as_deref())?;
    output::emit_json(&resolution)?;
    Ok(())
}
