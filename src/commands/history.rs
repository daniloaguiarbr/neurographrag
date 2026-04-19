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
    created_at_iso: String,
}

#[derive(Serialize)]
struct HistoryResponse {
    name: String,
    namespace: String,
    versions: Vec<HistoryVersion>,
    /// Tempo total de execução em milissegundos desde início do handler até serialização.
    elapsed_ms: u64,
}

pub fn run(args: HistoryArgs) -> Result<(), AppError> {
    let inicio = std::time::Instant::now();
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
            let created_at: i64 = r.get(8)?;
            let created_at_iso = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at, 0)
                .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
                .unwrap_or_default();
            Ok(HistoryVersion {
                version: r.get(0)?,
                name: r.get(1)?,
                memory_type: r.get(2)?,
                description: r.get(3)?,
                body: r.get(4)?,
                metadata: r.get(5)?,
                change_reason: r.get(6)?,
                changed_by: r.get(7)?,
                created_at,
                created_at_iso,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    output::emit_json(&HistoryResponse {
        name: args.name,
        namespace,
        versions,
        elapsed_ms: inicio.elapsed().as_millis() as u64,
    })?;

    Ok(())
}

#[cfg(test)]
mod testes {
    #[test]
    fn epoch_zero_gera_iso_valido() {
        let epoch: i64 = 0;
        let iso = chrono::DateTime::<chrono::Utc>::from_timestamp(epoch, 0)
            .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default();
        assert_eq!(iso, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn epoch_tipico_gera_iso_rfc3339() {
        let epoch: i64 = 1_745_000_000;
        let iso = chrono::DateTime::<chrono::Utc>::from_timestamp(epoch, 0)
            .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default();
        assert!(!iso.is_empty(), "created_at_iso não deve ser vazio");
        assert!(
            iso.ends_with('Z'),
            "created_at_iso deve terminar em Z (UTC)"
        );
        assert!(iso.contains('T'), "created_at_iso deve conter separador T");
    }

    #[test]
    fn epoch_negativo_retorna_string_vazia() {
        let epoch: i64 = i64::MIN;
        let iso = chrono::DateTime::<chrono::Utc>::from_timestamp(epoch, 0)
            .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default();
        assert_eq!(
            iso, "",
            "epoch inválido deve retornar string vazia via unwrap_or_default"
        );
    }
}
