pub mod chunking;
pub mod cli;
pub mod commands;
pub mod constants;
pub mod embedder;
pub mod errors;
pub mod graph;
pub mod namespace;
pub mod output;
pub mod paths;
pub mod pragmas;
pub mod storage;

mod embedded_migrations {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub use embedded_migrations::migrations;
