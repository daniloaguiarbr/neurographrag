// src/graph.rs

use crate::errors::AppError;
use rusqlite::{params, Connection};

/// BFS traversal from seed memories through entity relationships.
/// Returns memory_ids reachable via entity graph (excluding seeds).
pub fn traverse_from_memories(
    conn: &Connection,
    seed_memory_ids: &[i64],
    namespace: &str,
    min_weight: f64,
    max_hops: u32,
) -> Result<Vec<i64>, AppError> {
    if seed_memory_ids.is_empty() || max_hops == 0 {
        return Ok(vec![]);
    }

    // Step 1: collect seed entity IDs from seed memories
    let mut seed_entities: Vec<i64> = Vec::new();
    for &mem_id in seed_memory_ids {
        let mut stmt =
            conn.prepare_cached("SELECT entity_id FROM memory_entities WHERE memory_id = ?1")?;
        let ids: Vec<i64> = stmt
            .query_map(params![mem_id], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        seed_entities.extend(ids);
    }
    seed_entities.sort_unstable();
    seed_entities.dedup();

    if seed_entities.is_empty() {
        return Ok(vec![]);
    }

    // Step 2: BFS over relationships
    use std::collections::HashSet;
    let mut visited: HashSet<i64> = seed_entities.iter().cloned().collect();
    let mut frontier = seed_entities.clone();

    for _ in 0..max_hops {
        if frontier.is_empty() {
            break;
        }
        let mut next_frontier = Vec::new();

        for &entity_id in &frontier {
            let mut stmt = conn.prepare_cached(
                "SELECT target_id FROM relationships
                 WHERE source_id = ?1 AND weight >= ?2 AND namespace = ?3",
            )?;
            let neighbors: Vec<i64> = stmt
                .query_map(params![entity_id, min_weight, namespace], |r| r.get(0))?
                .filter_map(|r| r.ok())
                .filter(|id| !visited.contains(id))
                .collect();

            for id in neighbors {
                visited.insert(id);
                next_frontier.push(id);
            }
        }
        frontier = next_frontier;
    }

    // Step 3: find memories connected to traversed entities (excluding seeds)
    let seed_set: HashSet<i64> = seed_memory_ids.iter().cloned().collect();
    let graph_only_entities: Vec<i64> = visited
        .into_iter()
        .filter(|id| !seed_entities.contains(id))
        .collect();

    let mut result_ids: Vec<i64> = Vec::new();
    for &entity_id in &graph_only_entities {
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT me.memory_id
             FROM memory_entities me
             JOIN memories m ON m.id = me.memory_id
             WHERE me.entity_id = ?1 AND m.deleted_at IS NULL",
        )?;
        let mem_ids: Vec<i64> = stmt
            .query_map(params![entity_id], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .filter(|id| !seed_set.contains(id))
            .collect();
        result_ids.extend(mem_ids);
    }

    result_ids.sort_unstable();
    result_ids.dedup();
    Ok(result_ids)
}
