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
        cli::Commands::Sites(config) => match config {
            cli::SitesCommands::List  => cli::sites::list(&db).await,
            cli::SitesCommands::Get { site } => cli::sites::print(&db, &site).await,
            cli::SitesCommands::Add(args) => cli::sites::create(&db, args).await,
            cli::SitesCommands::Update(args) => cli::sites::update(&db, args).await,
            cli::SitesCommands::Remove { site } => cli::sites::delete(&db, &site).await,
        },
        cli::Commands::Moderators(moderators) => match moderators {
            cli::ModeratorsCommands::Add(args) => cli::moderators::create(&db, args).await,
            cli::ModeratorsCommands::List => cli::moderators::list(&db).await,
            cli::ModeratorsCommands::Remove { name } => cli::moderators::remove(&db, &name).await,
        },
    };

    Ok(())
}
