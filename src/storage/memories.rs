use crate::embedder::f32_to_bytes;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewMemory {
    pub namespace: String,
    pub name: String,
    pub memory_type: String,
    pub description: String,
    pub body: String,
    pub body_hash: String,
    pub session_id: Option<String>,
    pub source: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct MemoryRow {
    pub id: i64,
    pub namespace: String,
    pub name: String,
    pub memory_type: String,
    pub description: String,
    pub body: String,
    pub body_hash: String,
    pub session_id: Option<String>,
    pub source: String,
    pub metadata: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn find_by_name(
    conn: &Connection,
    namespace: &str,
    name: &str,
) -> Result<Option<(i64, i64, i64)>, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT m.id, m.updated_at, COALESCE(MAX(v.version), 0)
         FROM memories m
         LEFT JOIN memory_versions v ON v.memory_id = m.id
         WHERE m.namespace = ?1 AND m.name = ?2 AND m.deleted_at IS NULL
         GROUP BY m.id",
    )?;
    let result = stmt.query_row(params![namespace, name], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    });
    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn find_by_hash(
    conn: &Connection,
    namespace: &str,
    body_hash: &str,
) -> Result<Option<i64>, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id FROM memories WHERE namespace = ?1 AND body_hash = ?2 AND deleted_at IS NULL",
    )?;
    match stmt.query_row(params![namespace, body_hash], |r| r.get(0)) {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn insert(conn: &Connection, m: &NewMemory) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO memories (namespace, name, type, description, body, body_hash, session_id, source, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            m.namespace, m.name, m.memory_type, m.description, m.body,
            m.body_hash, m.session_id, m.source,
            serde_json::to_string(&m.metadata)?
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    m: &NewMemory,
    expected_updated_at: Option<i64>,
) -> Result<bool, AppError> {
    let affected = if let Some(ts) = expected_updated_at {
        conn.execute(
            "UPDATE memories SET type=?2, description=?3, body=?4, body_hash=?5,
             session_id=?6, source=?7, metadata=?8
             WHERE id=?1 AND updated_at=?9 AND deleted_at IS NULL",
            params![
                id,
                m.memory_type,
                m.description,
                m.body,
                m.body_hash,
                m.session_id,
                m.source,
                serde_json::to_string(&m.metadata)?,
                ts
            ],
        )?
    } else {
        conn.execute(
            "UPDATE memories SET type=?2, description=?3, body=?4, body_hash=?5,
             session_id=?6, source=?7, metadata=?8
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                m.memory_type,
                m.description,
                m.body,
                m.body_hash,
                m.session_id,
                m.source,
                serde_json::to_string(&m.metadata)?
            ],
        )?
    };
    Ok(affected == 1)
}

pub fn upsert_vec(
    conn: &Connection,
    memory_id: i64,
    namespace: &str,
    memory_type: &str,
    embedding: &[f32],
    name: &str,
    snippet: &str,
) -> Result<(), AppError> {
    // sqlite-vec virtual tables do not support INSERT OR REPLACE semantics.
    // Must delete the existing row first, then insert.
    conn.execute(
        "DELETE FROM vec_memories WHERE memory_id = ?1",
        params![memory_id],
    )?;
    conn.execute(
        "INSERT INTO vec_memories(memory_id, namespace, type, embedding, name, snippet)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            memory_id,
            namespace,
            memory_type,
            f32_to_bytes(embedding),
            name,
            snippet
        ],
    )?;
    Ok(())
}

pub fn delete_vec(conn: &Connection, memory_id: i64) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM vec_memories WHERE memory_id = ?1",
        params![memory_id],
    )?;
    Ok(())
}

pub fn read_by_name(
    conn: &Connection,
    namespace: &str,
    name: &str,
) -> Result<Option<MemoryRow>, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, namespace, name, type, description, body, body_hash,
                session_id, source, metadata, created_at, updated_at
         FROM memories WHERE namespace=?1 AND name=?2 AND deleted_at IS NULL",
    )?;
    match stmt.query_row(params![namespace, name], |r| {
        Ok(MemoryRow {
            id: r.get(0)?,
            namespace: r.get(1)?,
            name: r.get(2)?,
            memory_type: r.get(3)?,
            description: r.get(4)?,
            body: r.get(5)?,
            body_hash: r.get(6)?,
            session_id: r.get(7)?,
            source: r.get(8)?,
            metadata: r.get(9)?,
            created_at: r.get(10)?,
            updated_at: r.get(11)?,
        })
    }) {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn soft_delete(conn: &Connection, namespace: &str, name: &str) -> Result<bool, AppError> {
    let affected = conn.execute(
        "UPDATE memories SET deleted_at = unixepoch() WHERE namespace=?1 AND name=?2 AND deleted_at IS NULL",
        params![namespace, name],
    )?;
    Ok(affected == 1)
}

pub fn list(
    conn: &Connection,
    namespace: &str,
    memory_type: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<Vec<MemoryRow>, AppError> {
    if let Some(mt) = memory_type {
        let mut stmt = conn.prepare(
            "SELECT id, namespace, name, type, description, body, body_hash,
                    session_id, source, metadata, created_at, updated_at
             FROM memories WHERE namespace=?1 AND type=?2 AND deleted_at IS NULL
             ORDER BY updated_at DESC LIMIT ?3 OFFSET ?4",
        )?;
        let rows = stmt
            .query_map(params![namespace, mt, limit as i64, offset as i64], |r| {
                Ok(MemoryRow {
                    id: r.get(0)?,
                    namespace: r.get(1)?,
                    name: r.get(2)?,
                    memory_type: r.get(3)?,
                    description: r.get(4)?,
                    body: r.get(5)?,
                    body_hash: r.get(6)?,
                    session_id: r.get(7)?,
                    source: r.get(8)?,
                    metadata: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, namespace, name, type, description, body, body_hash,
                    session_id, source, metadata, created_at, updated_at
             FROM memories WHERE namespace=?1 AND deleted_at IS NULL
             ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt
            .query_map(params![namespace, limit as i64, offset as i64], |r| {
                Ok(MemoryRow {
                    id: r.get(0)?,
                    namespace: r.get(1)?,
                    name: r.get(2)?,
                    memory_type: r.get(3)?,
                    description: r.get(4)?,
                    body: r.get(5)?,
                    body_hash: r.get(6)?,
                    session_id: r.get(7)?,
                    source: r.get(8)?,
                    metadata: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

pub fn knn_search(
    conn: &Connection,
    embedding: &[f32],
    namespace: &str,
    memory_type: Option<&str>,
    k: usize,
) -> Result<Vec<(i64, f32)>, AppError> {
    let bytes = f32_to_bytes(embedding);
    if let Some(mt) = memory_type {
        let mut stmt = conn.prepare(
            "SELECT memory_id, distance FROM vec_memories
             WHERE embedding MATCH ?1 AND namespace = ?2 AND type = ?3
             ORDER BY distance LIMIT ?4",
        )?;
        let rows = stmt
            .query_map(params![bytes, namespace, mt, k as i64], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, f32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    } else {
        let mut stmt = conn.prepare(
            "SELECT memory_id, distance FROM vec_memories
             WHERE embedding MATCH ?1 AND namespace = ?2
             ORDER BY distance LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(params![bytes, namespace, k as i64], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, f32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

pub fn read_full(conn: &Connection, memory_id: i64) -> Result<Option<MemoryRow>, AppError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, namespace, name, type, description, body, body_hash,
                session_id, source, metadata, created_at, updated_at
         FROM memories WHERE id=?1 AND deleted_at IS NULL",
    )?;
    match stmt.query_row(params![memory_id], |r| {
        Ok(MemoryRow {
            id: r.get(0)?,
            namespace: r.get(1)?,
            name: r.get(2)?,
            memory_type: r.get(3)?,
            description: r.get(4)?,
            body: r.get(5)?,
            body_hash: r.get(6)?,
            session_id: r.get(7)?,
            source: r.get(8)?,
            metadata: r.get(9)?,
            created_at: r.get(10)?,
            updated_at: r.get(11)?,
        })
    }) {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn fts_search(
    conn: &Connection,
    query: &str,
    namespace: &str,
    memory_type: Option<&str>,
    limit: usize,
) -> Result<Vec<MemoryRow>, AppError> {
    let fts_query = format!("{query}*");
    if let Some(mt) = memory_type {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.namespace, m.name, m.type, m.description, m.body, m.body_hash,
                    m.session_id, m.source, m.metadata, m.created_at, m.updated_at
             FROM fts_memories fts
             JOIN memories m ON m.id = fts.rowid
             WHERE fts_memories MATCH ?1 AND m.namespace = ?2 AND m.type = ?3 AND m.deleted_at IS NULL
             ORDER BY rank LIMIT ?4",
        )?;
        let rows = stmt
            .query_map(params![fts_query, namespace, mt, limit as i64], |r| {
                Ok(MemoryRow {
                    id: r.get(0)?,
                    namespace: r.get(1)?,
                    name: r.get(2)?,
                    memory_type: r.get(3)?,
                    description: r.get(4)?,
                    body: r.get(5)?,
                    body_hash: r.get(6)?,
                    session_id: r.get(7)?,
                    source: r.get(8)?,
                    metadata: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    } else {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.namespace, m.name, m.type, m.description, m.body, m.body_hash,
                    m.session_id, m.source, m.metadata, m.created_at, m.updated_at
             FROM fts_memories fts
             JOIN memories m ON m.id = fts.rowid
             WHERE fts_memories MATCH ?1 AND m.namespace = ?2 AND m.deleted_at IS NULL
             ORDER BY rank LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(params![fts_query, namespace, limit as i64], |r| {
                Ok(MemoryRow {
                    id: r.get(0)?,
                    namespace: r.get(1)?,
                    name: r.get(2)?,
                    memory_type: r.get(3)?,
                    description: r.get(4)?,
                    body: r.get(5)?,
                    body_hash: r.get(6)?,
                    session_id: r.get(7)?,
                    source: r.get(8)?,
                    metadata: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
