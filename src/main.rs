mod db;
mod cli;
mod server;

#[actix_web::main]
async fn main() {
    env_logger::init();

    let args = cli::Cli::new();

    let db = sqlx::SqlitePool::connect_lazy(&args.db_uri()).expect("Can't connect to database");
    sqlx::migrate!().run(&db).await.expect("Couldn't migrate database");

    match args.command {
        cli::Commands::Server(server) => server::run(server, db).await,
        cli::Commands::Config(config) => {
            match config.command {
                None => cli::config::print(&db, config.get.site).await,
                Some(c) => match c {
                    cli::ConfigCommands::List => cli::config::list(&db).await,
                    cli::ConfigCommands::Set(args) => cli::config::create_or_update(&db, args).await,
                    cli::ConfigCommands::ResetSecret { site } => cli::config::reset_secret(&db, site).await
                }
            }
        },
        cli::Commands::Moderators(moderators) => {
            match moderators.command {
                None => cli::moderators::list(&db, moderators.site).await,
                Some(c) => match c {
                    cli::ModeratorsCommands::Add(args) => cli::moderators::create(&db, args, moderators.site).await,
                }
            }
        }
    }
}
