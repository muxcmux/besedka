// use besedka::{cli, server};
use besedka::cli;
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
        // cli::Commands::Server(config) => server::run(config, db).await?,
        cli::Commands::Server(config) => println!("bout to run server"),
        cli::Commands::Config(config) => match config {
            cli::ConfigCommands::List  => cli::config::list(&db).await,
            cli::ConfigCommands::Get { site } => cli::config::print(&db, &site).await,
            cli::ConfigCommands::Set(args) => cli::config::create_or_update(&db, args).await,
            cli::ConfigCommands::Remove { site } => cli::config::delete(&db, &site).await,
        },
        cli::Commands::Moderators(moderators) => match moderators {
            cli::ModeratorsCommands::Add(args) => cli::moderators::create(&db, args).await,
            cli::ModeratorsCommands::List => cli::moderators::list(&db).await,
            cli::ModeratorsCommands::Remove { name } => todo!(),
        },
    };

    Ok(())
}
