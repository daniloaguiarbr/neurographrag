use crate::errors::AppError;
use crate::output;
use crate::paths::AppPaths;
use crate::pragmas::apply_init_pragmas;
use crate::storage::connection::open_rw;
use serde::Serialize;

#[derive(clap::Args)]
pub struct InitArgs {
    #[arg(long, env = "NEUROGRAPHRAG_DB_PATH")]
    pub db: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub force: bool,
    /// Namespace inicial a resolver. Alinhado à documentação bilíngue que prevê `init --namespace`.
    /// Se fornecido, escreve `NEUROGRAPHRAG_NAMESPACE` no arquivo `.neurographrag/config.toml`
    /// do diretório atual; caso contrário, resolve via env ou fallback `global`.
    #[arg(long)]
    pub namespace: Option<String>,
}

#[derive(Serialize)]
struct InitResponse {
    db_path: String,
    schema_version: String,
    model: String,
    dim: usize,
    /// Namespace ativo resolvido durante a inicialização, alinhado à doc bilíngue.
    namespace: String,
    status: String,
    /// Tempo total de execução em milissegundos desde início do handler até serialização.
    elapsed_ms: u64,
}

pub fn run(args: InitArgs) -> Result<(), AppError> {
    let inicio = std::time::Instant::now();
    let paths = AppPaths::resolve(args.db.as_deref())?;
    paths.ensure_dirs()?;

    let namespace = crate::namespace::resolve_namespace(args.namespace.as_deref())?;

    let mut conn = open_rw(&paths.db)?;

    apply_init_pragmas(&conn)?;

    crate::migrations::runner()
        .run(&mut conn)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("migration failed: {e}")))?;

    let schema_version = latest_schema_version(&conn)?;

    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('schema_version', ?1)",
        rusqlite::params![schema_version],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('model', 'multilingual-e5-small')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('dim', '384')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('created_at', CAST(unixepoch() AS TEXT))",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('neurographrag_version', ?1)",
        rusqlite::params![crate::constants::NEUROGRAPHRAG_VERSION],
    )?;

    output::emit_progress_i18n(
        "Initializing embedding model (may download on first run)...",
        "Inicializando modelo de embedding (pode baixar na primeira execução)...",
    );

    let embedder = crate::embedder::get_embedder(&paths.models)?;
    let test_emb = crate::embedder::embed_passage(embedder, "smoke test")?;

    output::emit_json(&InitResponse {
        db_path: paths.db.display().to_string(),
        schema_version,
        model: "multilingual-e5-small".to_string(),
        dim: test_emb.len(),
        namespace,
        status: "ok".to_string(),
        elapsed_ms: inicio.elapsed().as_millis() as u64,
    })?;

    Ok(())
}

fn latest_schema_version(conn: &rusqlite::Connection) -> Result<String, AppError> {
    match conn.query_row(
        "SELECT version FROM refinery_schema_history ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(version) => Ok(version.to_string()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok("0".to_string()),
        Err(err) => Err(AppError::Database(err)),
    }
}
