pub const EMBEDDING_DIM: usize = 384;
pub const FASTEMBED_MODEL_DEFAULT: &str = "multilingual-e5-small";
pub const FASTEMBED_BATCH_SIZE: usize = 32;

pub const MAX_MEMORY_NAME_LEN: usize = 80;
pub const MAX_MEMORY_DESCRIPTION_LEN: usize = 500;
pub const MAX_MEMORY_BODY_LEN: usize = 20_000;
pub const MAX_BODY_CHARS_BEFORE_CHUNK: usize = 8_000;

pub const MAX_SQLITE_BUSY_RETRIES: u32 = 5;
pub const QUERY_TIMEOUT_MILLIS: u64 = 5_000;

pub const DEDUP_FUZZY_THRESHOLD: f64 = 0.8;
pub const DEDUP_SEMANTIC_THRESHOLD: f32 = 0.1;

pub const MAX_GRAPH_HOPS: u32 = 2;
pub const MIN_RELATION_WEIGHT: f64 = 0.3;

pub const K_MEMORIES_DEFAULT: usize = 10;
pub const K_ENTITIES_SEARCH: usize = 5;
pub const MAX_ENTITIES_PER_MEMORY: usize = 30;
pub const MAX_RELATIONSHIPS_PER_MEMORY: usize = 50;

pub const BUSY_TIMEOUT_MILLIS: i32 = 5_000;
pub const CACHE_SIZE_KB: i32 = -64_000;
pub const MMAP_SIZE_BYTES: i64 = 268_435_456;
pub const WAL_AUTOCHECKPOINT_PAGES: i32 = 1_000;

pub const RRF_K_DEFAULT: u32 = 60;
pub const CHUNK_SIZE_TOKENS: usize = 400;
pub const CHUNK_OVERLAP_TOKENS: usize = 50;
pub const DAEMON_PING_TIMEOUT_MS: u64 = 10;
pub const DAEMON_IDLE_SHUTDOWN_SECS: u64 = 600;

pub const PASSAGE_PREFIX: &str = "passage: ";
pub const QUERY_PREFIX: &str = "query: ";

pub const NEUROGRAPHRAG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const SLUG_REGEX: &str = r"^[a-z0-9]+(?:-[a-z0-9]+)*$";
