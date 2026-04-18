use clap::Parser;
use neurographrag::{cli::Cli, commands, storage::connection::register_vec_extension};

fn main() {
    let log_level = std::env::var("NEUROGRAPHRAG_LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());
    let log_format =
        std::env::var("NEUROGRAPHRAG_LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    if log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(tracing_subscriber::EnvFilter::new(&log_level))
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(&log_level))
            .with_writer(std::io::stderr)
            .init();
    }

    register_vec_extension();

    let cli = Cli::parse();
    let result = match cli.command {
        neurographrag::cli::Commands::Init(args) => commands::init::run(args),
        neurographrag::cli::Commands::Remember(args) => commands::remember::run(args),
        neurographrag::cli::Commands::Recall(args) => commands::recall::run(args),
        neurographrag::cli::Commands::Read(args) => commands::read::run(args),
        neurographrag::cli::Commands::List(args) => commands::list::run(args),
        neurographrag::cli::Commands::Forget(args) => commands::forget::run(args),
        neurographrag::cli::Commands::Purge(args) => commands::purge::run(args),
        neurographrag::cli::Commands::Rename(args) => commands::rename::run(args),
        neurographrag::cli::Commands::Edit(args) => commands::edit::run(args),
        neurographrag::cli::Commands::History(args) => commands::history::run(args),
        neurographrag::cli::Commands::Restore(args) => commands::restore::run(args),
        neurographrag::cli::Commands::HybridSearch(args) => commands::hybrid_search::run(args),
        neurographrag::cli::Commands::Health(args) => commands::health::run(args),
        neurographrag::cli::Commands::Migrate(args) => commands::migrate::run(args),
        neurographrag::cli::Commands::NamespaceDetect(args) => {
            commands::namespace_detect::run(args)
        }
        neurographrag::cli::Commands::Optimize(args) => commands::optimize::run(args),
        neurographrag::cli::Commands::Stats(args) => commands::stats::run(args),
        neurographrag::cli::Commands::SyncSafeCopy(args) => commands::sync_safe_copy::run(args),
        neurographrag::cli::Commands::Vacuum(args) => commands::vacuum::run(args),
    };

    if let Err(e) = result {
        tracing::error!(error = %e);
        eprintln!("Error: {e}");
        std::process::exit(e.exit_code());
    }
}
