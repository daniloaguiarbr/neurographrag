-- FTS5 sync triggers (vec_memories synced in Rust because requires embedding bytes)
-- trg_fts_au (AFTER UPDATE) is intentionally absent: sqlite-vec loaded via
-- sqlite3_auto_extension conflicts with FTS5 inside AFTER UPDATE triggers on memories.
-- FTS5 sync for edit/rename/restore is handled explicitly in Rust handlers.

CREATE TRIGGER IF NOT EXISTS trg_fts_ai
AFTER INSERT ON memories WHEN NEW.deleted_at IS NULL BEGIN
    INSERT INTO fts_memories(rowid, name, description, body)
    VALUES (NEW.id, NEW.name, NEW.description, NEW.body);
END;

CREATE TRIGGER IF NOT EXISTS trg_fts_ad
AFTER DELETE ON memories BEGIN
    INSERT INTO fts_memories(fts_memories, rowid, name, description, body)
    VALUES('delete', OLD.id, OLD.name, OLD.description, OLD.body);
END;

-- FTS soft-delete sync is handled in Rust (forget.rs) to avoid sqlite-vec
-- extension interference when running DELETE on FTS5 inside a trigger.
