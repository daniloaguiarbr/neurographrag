use crate::errors::AppError;
use serde::Serialize;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Text,
    Markdown,
}

pub fn emit_json<T: Serialize>(value: &T) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}

pub fn emit_json_compact<T: Serialize>(value: &T) -> Result<(), AppError> {
    let json = serde_json::to_string(value)?;
    println!("{json}");
    Ok(())
}

pub fn emit_text(msg: &str) {
    println!("{msg}");
}

pub fn emit_progress(msg: &str) {
    eprintln!("{msg}");
}

#[derive(Serialize)]
pub struct RememberResponse {
    pub memory_id: i64,
    pub name: String,
    pub action: String,
    pub version: i64,
    pub entities_persisted: usize,
    pub relationships_persisted: usize,
    pub chunks_created: usize,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct RecallItem {
    pub memory_id: i64,
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub description: String,
    pub snippet: String,
    pub distance: f32,
    pub source: String,
}

#[derive(Serialize)]
pub struct RecallResponse {
    pub query: String,
    pub direct_matches: Vec<RecallItem>,
    pub graph_matches: Vec<RecallItem>,
}
