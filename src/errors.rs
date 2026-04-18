use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("duplicate detected: {0}")]
    Duplicate(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("namespace not resolved: {0}")]
    NamespaceError(String),

    #[error("limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("embedding error: {0}")]
    Embedding(String),

    #[error("sqlite-vec extension failed: {0}")]
    VecExtension(String),

    #[error("database busy: {0}")]
    DbBusy(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 1,
            Self::Duplicate(_) => 2,
            Self::Conflict(_) => 3,
            Self::NotFound(_) => 4,
            Self::NamespaceError(_) => 5,
            Self::LimitExceeded(_) => 6,
            Self::Database(_) => 10,
            Self::Embedding(_) => 11,
            Self::VecExtension(_) => 12,
            Self::DbBusy(_) => 13,
            Self::Io(_) => 14,
            Self::Internal(_) => 20,
            Self::Json(_) => 20,
        }
    }
}
