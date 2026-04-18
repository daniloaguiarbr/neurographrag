// src/chunking.rs
// Token-based chunking for E5 model (512 token limit)

use crate::constants::{CHUNK_OVERLAP_TOKENS, CHUNK_SIZE_TOKENS, EMBEDDING_DIM};

const CHARS_PER_TOKEN: usize = 4;
pub const CHUNK_SIZE_CHARS: usize = CHUNK_SIZE_TOKENS * CHARS_PER_TOKEN;
pub const CHUNK_OVERLAP_CHARS: usize = CHUNK_OVERLAP_TOKENS * CHARS_PER_TOKEN;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub token_count_approx: usize,
}

pub fn needs_chunking(body: &str) -> bool {
    body.len() > CHUNK_SIZE_CHARS
}

pub fn split_into_chunks(body: &str) -> Vec<Chunk> {
    if !needs_chunking(body) {
        return vec![Chunk {
            token_count_approx: body.len() / CHARS_PER_TOKEN,
            text: body.to_string(),
            start_offset: 0,
            end_offset: body.len(),
        }];
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;

    while start < body.len() {
        let desired_end = (start + CHUNK_SIZE_CHARS).min(body.len());
        let end = if desired_end < body.len() {
            find_split_boundary(body, start, desired_end)
        } else {
            desired_end
        };

        let text = body[start..end].to_string();
        let token_count_approx = text.len() / CHARS_PER_TOKEN;
        chunks.push(Chunk {
            text,
            start_offset: start,
            end_offset: end,
            token_count_approx,
        });

        if end >= body.len() {
            break;
        }
        start = end.saturating_sub(CHUNK_OVERLAP_CHARS);
    }

    chunks
}

fn find_split_boundary(body: &str, start: usize, desired_end: usize) -> usize {
    let slice = &body[start..desired_end];
    if let Some(pos) = slice.rfind("\n\n") {
        return start + pos + 2;
    }
    if let Some(pos) = slice.rfind(". ") {
        return start + pos + 2;
    }
    if let Some(pos) = slice.rfind(' ') {
        return start + pos + 1;
    }
    desired_end
}

pub fn aggregate_embeddings(chunk_embeddings: &[Vec<f32>]) -> Vec<f32> {
    if chunk_embeddings.is_empty() {
        return vec![0.0f32; EMBEDDING_DIM];
    }
    if chunk_embeddings.len() == 1 {
        return chunk_embeddings[0].clone();
    }

    let dim = chunk_embeddings[0].len();
    let mut mean = vec![0.0f32; dim];
    for emb in chunk_embeddings {
        for (i, v) in emb.iter().enumerate() {
            mean[i] += v;
        }
    }
    let n = chunk_embeddings.len() as f32;
    for v in &mut mean {
        *v /= n;
    }

    let norm: f32 = mean.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-9 {
        for v in &mut mean {
            *v /= norm;
        }
    }
    mean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_body_no_chunking() {
        let body = "short text";
        assert!(!needs_chunking(body));
        let chunks = split_into_chunks(body);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, body);
    }

    #[test]
    fn test_long_body_produces_multiple_chunks() {
        let body = "word ".repeat(1000);
        assert!(needs_chunking(&body));
        let chunks = split_into_chunks(&body);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_aggregate_embeddings_normalizes() {
        let embs = vec![vec![1.0f32, 0.0], vec![0.0f32, 1.0]];
        let agg = aggregate_embeddings(&embs);
        let norm: f32 = agg.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }
}
