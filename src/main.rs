// Standard library imports
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use clap::{self, Parser};
use clients::{file_system::FileSystemClient, git::GitClient};

// Local module imports
use dotty::cli::Cli;
use dotty::config::{ConfigLoaderClient, TomlConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let fs = FileSystemClient;
    let loader = ConfigLoaderClient;
    let git = Arc::new(GitClient);

    let config = TomlConfig::from_path_or_default(&fs, &loader).await?;
    config
        .configure_logging(ConfigLoaderClient::is_running_under_cargo)
        .await?;

    let cli = Cli::parse();

    cli.command.execute(config, &fs, &loader, git).await?;

    Ok(())
}
