use crate::commands::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "neurographrag")]
#[command(version)]
#[command(about = "Local GraphRAG memory for LLMs in a single SQLite file")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize database and download embedding model
    Init(init::InitArgs),
    /// Save a memory with optional entity graph
    Remember(remember::RememberArgs),
    /// Search memories semantically
    Recall(recall::RecallArgs),
    /// Read a memory by exact name
    Read(read::ReadArgs),
    /// List memories with filters
    List(list::ListArgs),
    /// Soft-delete a memory
    Forget(forget::ForgetArgs),
    /// Permanently delete soft-deleted memories
    Purge(purge::PurgeArgs),
    /// Rename a memory preserving history
    Rename(rename::RenameArgs),
    /// Edit a memory's body or description
    Edit(edit::EditArgs),
    /// List all versions of a memory
    History(history::HistoryArgs),
    /// Restore a memory to a previous version
    Restore(restore::RestoreArgs),
    /// Search using hybrid vector + full-text search
    HybridSearch(hybrid_search::HybridSearchArgs),
    /// Show database health
    Health(health::HealthArgs),
    /// Apply pending schema migrations
    Migrate(migrate::MigrateArgs),
    /// Resolve namespace precedence for the current invocation
    NamespaceDetect(namespace_detect::NamespaceDetectArgs),
    /// Run PRAGMA optimize on the database
    Optimize(optimize::OptimizeArgs),
    /// Show database statistics
    Stats(stats::StatsArgs),
    /// Create a checkpointed copy safe for file sync
    SyncSafeCopy(sync_safe_copy::SyncSafeCopyArgs),
    /// Run VACUUM after checkpointing the WAL
    Vacuum(vacuum::VacuumArgs),
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
    Decision,
    Incident,
    Skill,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Feedback => "feedback",
            Self::Project => "project",
            Self::Reference => "reference",
            Self::Decision => "decision",
            Self::Incident => "incident",
            Self::Skill => "skill",
        }
    }
}
