use crate::constants::{EMBEDDING_DIM, FASTEMBED_BATCH_SIZE, PASSAGE_PREFIX, QUERY_PREFIX};
use crate::errors::AppError;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::Path;

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    pub fn new(models_dir: &Path) -> Result<Self, AppError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::MultilingualE5Small)
                .with_show_download_progress(true)
                .with_cache_dir(models_dir.to_path_buf()),
        )
        .map_err(|e| AppError::Embedding(e.to_string()))?;
        Ok(Self { model })
    }

    pub fn embed_passage(&mut self, text: &str) -> Result<Vec<f32>, AppError> {
        let prefixed = format!("{PASSAGE_PREFIX}{text}");
        let results = self
            .model
            .embed(vec![prefixed.as_str()], Some(1))
            .map_err(|e| AppError::Embedding(e.to_string()))?;
        let emb = results
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Embedding("empty embedding result".into()))?;
        assert_eq!(emb.len(), EMBEDDING_DIM, "unexpected embedding dimension");
        Ok(emb)
    }

    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>, AppError> {
        let prefixed = format!("{QUERY_PREFIX}{text}");
        let results = self
            .model
            .embed(vec![prefixed.as_str()], Some(1))
            .map_err(|e| AppError::Embedding(e.to_string()))?;
        let emb = results
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Embedding("empty embedding result".into()))?;
        Ok(emb)
    }

    pub fn embed_passages_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        let prefixed: Vec<String> = texts
            .iter()
            .map(|t| format!("{PASSAGE_PREFIX}{t}"))
            .collect();
        let strs: Vec<&str> = prefixed.iter().map(String::as_str).collect();
        let results = self
            .model
            .embed(strs, Some(FASTEMBED_BATCH_SIZE))
            .map_err(|e| AppError::Embedding(e.to_string()))?;
        for emb in &results {
            assert_eq!(emb.len(), EMBEDDING_DIM, "unexpected embedding dimension");
        }
        Ok(results)
    }
}

/// Convert &[f32] to &[u8] for sqlite-vec storage.
/// # Safety
/// Safe because f32 has no padding and is well-defined bit pattern.
pub fn f32_to_bytes(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}
