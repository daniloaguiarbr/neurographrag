-- sqlite-vec must be loaded before this migration runs
-- partition key columns enable filtered KNN (max 4 partition keys)

CREATE VIRTUAL TABLE IF NOT EXISTS vec_memories USING vec0(
    memory_id  INTEGER PRIMARY KEY,
    namespace  TEXT partition key,
    type       TEXT partition key,
    embedding  float[384] distance_metric=cosine,
    +name      TEXT,
    +snippet   TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS vec_entities USING vec0(
    entity_id  INTEGER PRIMARY KEY,
    namespace  TEXT partition key,
    type       TEXT partition key,
    embedding  float[384] distance_metric=cosine,
    +name      TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(
    rowid      INTEGER PRIMARY KEY,
    memory_id  INTEGER partition key,
    chunk_idx  INTEGER,
    embedding  float[384] distance_metric=cosine
);
