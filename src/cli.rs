pub mod sites;
pub mod moderators;

use clap::{Parser, Subcommand, Args};
use std::net::SocketAddr;

#[derive(Parser, Debug, Clone)]
#[command(name = "besedka", author, version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, value_name = "FILE", default_value = "besedka.sqlite", global = true)]
    /// Database file
    pub db: String
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn db_uri(&self) -> String {
        format!("sqlite:{}?mode=rwc", self.db)
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    #[command(aliases(["s", "run"]))]
    Server(ServerArgs),
    #[command(subcommand)]
    #[command(alias("site"))]
    Sites(SitesCommands),
    #[command(subcommand)]
    #[command(alias("moderator"))]
    Moderators(ModeratorsCommands),
}

#[derive(Debug, Clone, Args)]
/// Run the besedka commenting system server
pub struct ServerArgs {
    #[arg(short, long, value_name = "ADDR", default_value = "0.0.0.0:6353")]
    /// Address and port to listen on
    pub bind: SocketAddr,

    #[arg(long, value_name = "FILE", value_parser = valid_file)]
    /// Path to a key file (required for TLS)
    pub ssl_key: Option<String>,

    #[arg(long, value_name = "FILE", value_parser = valid_file)]
    /// Path to a certificate pem (required for TLS)
    pub ssl_cert: Option<String>
}

#[derive(Debug, Clone, Subcommand)]
/// View or edit site configuration
pub enum SitesCommands {
    /// View all available configurations
    List,
    /// Display a site config
    #[command(alias("show"))]
    Get { site: String },
    /// Delete a site configuration.
    /// This will NOT remove comments or pages
    /// assiciated to the site
    #[command(alias("delete"))]
    Remove { site: String },
    #[command(alias("create"))]
    /// Add a site config
    Add(SitesCommandArgs),
    #[command(alias("edit"))]
    /// Update a site config
    Update(SitesCommandArgs),
}

#[derive(Debug, Clone, Args)]
pub struct SitesCommandArgs {
    pub site: String,

    #[arg(long)]
    /// Set to true to restrict reading and writing comments
    /// only to authorised users or logged in moderators
    pub private: Option<bool>,

    #[arg(long)]
    /// Set to true to allow anyone to post comments
    pub anonymous: Option<bool>,

    #[arg(long)]
    /// Set to true to require moderator approval
    /// before comments are visible to everyone
    pub moderated: Option<bool>,
}

#[derive(Debug, Clone, Subcommand)]
/// Manage moderators
pub enum ModeratorsCommands {
    /// List all moderators
    List,
    #[command(alias("create"))]
    /// Create a moderator
    Add(ModeratorsAddCommandArgs),
    /// Remove a moderator
    #[command(alias("delete"))]
    Remove { name: String },
    /// Update a moderator
    #[command(alias("edit"))]
    Update(ModeratorsUpdateCommandArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ModeratorsAddCommandArgs {
    /// Name to log in with also displayed in comments, must be unique
    pub name: String,
    /// Password used for login
    pub password: String,
    #[arg(short, long)]
    /// Avatar - any valid src value for an img tag
    pub avatar: Option<String>,
    #[arg(long)]
    /// Is this moderator also an OP
    pub op: Option<bool>,
}

#[derive(Debug, Clone, Args)]
pub struct ModeratorsUpdateCommandArgs {
    /// Name of the moderator to update
    pub name: String,
    #[arg(short, long)]
    /// Password used for login
    pub password: Option<String>,
    #[arg(short, long)]
    /// Avatar - any valid src value for an img tag
    pub avatar: Option<String>,
    #[arg(long)]
    /// Is this moderator also an OP
    pub op: Option<bool>,
}

fn valid_file(s: &str) -> Result<String, anyhow::Error> {
    let file = std::path::PathBuf::from(s);
    if file.is_file() {
        return Ok(String::from(s))
    }

    anyhow::bail!("File does not exist")
}
