-- External content FTS5 table backed by memories
CREATE VIRTUAL TABLE IF NOT EXISTS fts_memories USING fts5(
    name,
    description,
    body,
    content='memories',
    content_rowid='id',
    tokenize='unicode61 remove_diacritics 1'
);
