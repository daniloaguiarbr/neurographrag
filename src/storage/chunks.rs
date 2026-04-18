// src/storage/chunks.rs
// Chunk storage for bodies exceeding 512 tokens E5 limit

use crate::embedder::f32_to_bytes;
use crate::errors::AppError;
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub memory_id: i64,
    pub chunk_idx: i32,
    pub chunk_text: String,
    pub start_offset: i32,
    pub end_offset: i32,
    pub token_count: i32,
}

pub fn insert_chunks(conn: &Connection, chunks: &[Chunk]) -> Result<(), AppError> {
    for chunk in chunks {
        conn.execute(
            "INSERT INTO memory_chunks (memory_id, chunk_idx, chunk_text, start_offset, end_offset, token_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                chunk.memory_id,
                chunk.chunk_idx,
                chunk.chunk_text,
                chunk.start_offset,
                chunk.end_offset,
                chunk.token_count,
            ],
        )?;
    }
    Ok(())
}

pub fn upsert_chunk_vec(
    conn: &Connection,
    _rowid: i64,
    memory_id: i64,
    chunk_idx: i32,
    embedding: &[f32],
) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO vec_chunks(rowid, memory_id, chunk_idx, embedding)
         VALUES (
             (SELECT id FROM memory_chunks WHERE memory_id = ?1 AND chunk_idx = ?2),
             ?1, ?2, ?3
         )",
        params![memory_id, chunk_idx, f32_to_bytes(embedding)],
    )?;
    Ok(())
}

pub fn delete_chunks(conn: &Connection, memory_id: i64) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM memory_chunks WHERE memory_id = ?1",
        params![memory_id],
    )?;
    Ok(())
}

pub fn knn_search_chunks(
    conn: &Connection,
    embedding: &[f32],
    k: usize,
) -> Result<Vec<(i64, i32, f32)>, AppError> {
    let bytes = f32_to_bytes(embedding);
    let mut stmt = conn.prepare(
        "SELECT memory_id, chunk_idx, distance FROM vec_chunks
         WHERE embedding MATCH ?1
         ORDER BY distance LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![bytes, k as i64], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i32>(1)?,
                r.get::<_, f32>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_chunks_by_memory(conn: &Connection, memory_id: i64) -> Result<Vec<Chunk>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT memory_id, chunk_idx, chunk_text, start_offset, end_offset, token_count
         FROM memory_chunks WHERE memory_id = ?1 ORDER BY chunk_idx",
    )?;
    let rows = stmt
        .query_map(params![memory_id], |r| {
            Ok(Chunk {
                memory_id: r.get(0)?,
                chunk_idx: r.get(1)?,
                chunk_text: r.get(2)?,
                start_offset: r.get(3)?,
                end_offset: r.get(4)?,
                token_count: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
