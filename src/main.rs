use besedka::{cli, server};
use sqlx::{migrate, SqlitePool};

use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = cli::Cli::new();

    let db = SqlitePool::connect_lazy(&args.db_uri()).expect("Can't connect to database");
    migrate!()
        .run(&db)
        .await
        .expect("Couldn't migrate database");

    match args.command {
        cli::Commands::Server(config) => server::run(config, db).await?,
        cli::Commands::Config(config) => match config.command {
            None => cli::config::print(&db, config.get.site).await,
            Some(c) => match c {
                cli::ConfigCommands::List => cli::config::list(&db).await,
                cli::ConfigCommands::Set(args) => cli::config::create_or_update(&db, args).await,
                cli::ConfigCommands::ResetSecret { site } => {
                    cli::config::reset_secret(&db, site).await
                }
            },
        },
        cli::Commands::Moderators(moderators) => match moderators.command {
            None => cli::moderators::list(&db, moderators.site).await,
            Some(c) => match c {
                cli::ModeratorsCommands::Add(args) => {
                    cli::moderators::create(&db, args, moderators.site).await
                }
            },
        },
    };

    Ok(())
}
