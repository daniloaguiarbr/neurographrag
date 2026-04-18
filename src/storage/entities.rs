use crate::embedder::f32_to_bytes;
use crate::errors::AppError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewEntity {
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewRelationship {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub strength: f64,
    pub description: Option<String>,
}

pub fn upsert_entity(conn: &Connection, namespace: &str, e: &NewEntity) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO entities (namespace, name, type, description)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(namespace, name) DO UPDATE SET
           type        = excluded.type,
           description = COALESCE(excluded.description, entities.description),
           updated_at  = unixepoch()",
        params![namespace, e.name, e.entity_type, e.description],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM entities WHERE namespace = ?1 AND name = ?2",
        params![namespace, e.name],
        |r| r.get(0),
    )?;
    Ok(id)
}

pub fn upsert_entity_vec(
    conn: &Connection,
    entity_id: i64,
    namespace: &str,
    entity_type: &str,
    embedding: &[f32],
    name: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO vec_entities(entity_id, namespace, type, embedding, name)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            entity_id,
            namespace,
            entity_type,
            f32_to_bytes(embedding),
            name
        ],
    )?;
    Ok(())
}

pub fn upsert_relationship(
    conn: &Connection,
    namespace: &str,
    source_id: i64,
    target_id: i64,
    rel: &NewRelationship,
) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO relationships (namespace, source_id, target_id, relation, weight, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(source_id, target_id, relation) DO UPDATE SET
           weight = excluded.weight,
           description = COALESCE(excluded.description, relationships.description)",
        params![
            namespace,
            source_id,
            target_id,
            rel.relation,
            rel.strength,
            rel.description
        ],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM relationships WHERE source_id=?1 AND target_id=?2 AND relation=?3",
        params![source_id, target_id, rel.relation],
        |r| r.get(0),
    )?;
    Ok(id)
}

pub fn link_memory_entity(
    conn: &Connection,
    memory_id: i64,
    entity_id: i64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO memory_entities (memory_id, entity_id) VALUES (?1, ?2)",
        params![memory_id, entity_id],
    )?;
    Ok(())
}

pub fn link_memory_relationship(
    conn: &Connection,
    memory_id: i64,
    rel_id: i64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR IGNORE INTO memory_relationships (memory_id, relationship_id) VALUES (?1, ?2)",
        params![memory_id, rel_id],
    )?;
    Ok(())
}

pub fn increment_degree(conn: &Connection, entity_id: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE entities SET degree = degree + 1 WHERE id = ?1",
        params![entity_id],
    )?;
    Ok(())
}

pub fn knn_search(
    conn: &Connection,
    embedding: &[f32],
    namespace: &str,
    k: usize,
) -> Result<Vec<(i64, f32)>, AppError> {
    let bytes = f32_to_bytes(embedding);
    let mut stmt = conn.prepare(
        "SELECT entity_id, distance FROM vec_entities
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
