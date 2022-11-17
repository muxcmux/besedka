pub mod config;
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
    Server(Server),
    Config(Config),
    Moderators(Moderators)
}

#[derive(Debug, Clone, Args)]
/// Run the besedka commenting system server
pub struct Server {
    #[arg(short, long, value_name = "ADDR", default_value = "0.0.0.0:6353")]
    /// Address and port to listen on
    pub bind: SocketAddr,

    #[arg(short, long, value_name = "FILE", value_parser = valid_file)]
    /// Path to a key file (required for TLS)
    pub ssl_key: Option<String>,

    #[arg(long, value_name = "FILE", value_parser = valid_file)]
    /// Path to a certificate pem (required for TLS)
    pub ssl_cert: Option<String>
}

#[derive(Debug, Clone, Args)]
/// View or edit besedka configuration
pub struct Config {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
    #[command(flatten)]
    pub get: ConfigGetCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommands {
    /// Prints all available configurations
    List,
    Set(ConfigSetCommand),
    /// Generates a new secret for the configuration
    ResetSecret {
        #[arg(short, long, value_name = "HOST", default_value = "default")]
        /// Hostname (including subdomain)
        site: String
    }
}

#[derive(Debug, Clone, Args)]
#[command(args_conflicts_with_subcommands = true)]
/// Print the configuration for a site
pub struct ConfigGetCommand {
    #[arg(short, long, value_name = "HOST", default_value = "default")]
    /// Hostname (including subdomain)
    pub site: String,
}

#[derive(Debug, Clone, Args)]
/// Create or update a config for a site
pub struct ConfigSetCommand {
    #[arg(short, long, value_name = "HOST", default_value = "default")]
    /// Hostname (including subdomain)
    pub site: String,

    #[arg(long)]
    /// Set to true to restrict reading and writing comments
    /// only to authorised users or logged in moderators
    pub private: Option<bool>,

    #[arg(long)]
    /// Set to true to allow anyone to post comments
    pub anonymous_comments: Option<bool>,

    #[arg(long)]
    /// Set true to require a moderator to approve comments
    /// before they are visible to everyone on your page
    pub moderated: Option<bool>,

    #[arg(long)]
    /// Number of comments to load at once
    pub comments_per_page: Option<u32>,

    #[arg(long)]
    /// Number of replies to load at once for each comment
    pub replies_per_comment: Option<u32>,

    #[arg(long)]
    /// Minutes before a comment can no longer be edited.
    /// Setting this to 0 will disable authors from editing their comments completely
    pub minutes_to_edit: Option<u32>,

    #[arg(long)]
    /// The theme of the comment widget
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Args)]
/// View or edit moderators
pub struct Moderators {
    #[arg(short, long, value_name = "HOST", default_value = "default", global = true)]
    /// Hostname (including subdomain)
    pub site: String,
    #[command(subcommand)]
    pub command: Option<ModeratorsCommands>,
    #[command(flatten)]
    pub list: ModeratorsListCommand
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModeratorsCommands {
    Add(ModeratorsAddCommand),
}

#[derive(Debug, Clone, Args)]
#[command(args_conflicts_with_subcommands = true)]
/// Prints all the moderators for a site
pub struct ModeratorsListCommand;

#[derive(Debug, Clone, Args)]
pub struct ModeratorsAddCommand {
    #[arg[short, long]]
    /// The moderator username used to log in
    pub username: String,
    #[arg(short, long)]
    /// Name displayed in comments
    pub name: String,
    #[arg(short, long)]
    /// Password used for login
    pub password: String,
    #[arg(short, long)]
    /// Avatar - any valid src value for an img tag
    pub avatar: Option<String>
}

fn valid_file(s: &str) -> Result<String, anyhow::Error> {
    let file = std::path::PathBuf::from(s);
    if file.is_file() {
        return Ok(String::from(s))
    }

    anyhow::bail!("File does not exist")
}
