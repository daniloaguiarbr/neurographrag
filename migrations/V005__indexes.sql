CREATE INDEX IF NOT EXISTS idx_relationships_source_id ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target_id ON relationships(target_id);
CREATE INDEX IF NOT EXISTS idx_relationships_namespace_relation ON relationships(namespace, relation);
CREATE INDEX IF NOT EXISTS idx_entities_namespace_degree ON entities(namespace, degree DESC);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_memory_id ON memory_chunks(memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_relationships_relationship_id ON memory_relationships(relationship_id);
