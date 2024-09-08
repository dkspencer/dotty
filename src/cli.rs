// Standard library imports
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use clap::{Parser, Subcommand};
use clients::{file_system::FileSystem, git::Git};

// Local module imports
use crate::{
    config::{command::Commands as ConfigCommands, ConfigLoader, TomlConfig},
    ui::cli::style,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(propagate_version = true, infer_subcommands = true)]
#[clap(styles = style())]
pub struct Cli {
    #[command(subcommand)]
    pub command: DottyCommands,
}

#[derive(Debug, Subcommand)]
pub enum DottyCommands {
    #[command(subcommand)]
    Config(ConfigCommands),
}

impl DottyCommands {
    /// Executes the selected Dotty command.
    ///
    /// This function serves as a dispatcher for the various subcommands of Dotty.
    /// It matches on the enum variant of `DottyCommands` and calls the appropriate
    /// `execute` method for the selected subcommand.
    ///
    /// # Arguments
    /// * `self` - The `DottyCommands` enum instance representing the selected subcommand.
    /// * `config` - The `TomlConfig` instance containing the current configuration.
    /// * `fs` - A reference to an implementation of `FileSystem` for file operations.
    /// * `loader` - A reference to an implementation of `ConfigLoader` for config serialization/deserialization.
    /// * `git` - An implementation of Git for interacting with the Git API.
    ///
    /// # Returns
    /// Returns `Ok(())` if the command executes successfully, or an error if the command fails.
    ///
    /// # Errors
    /// This function may return an error if:
    /// - The selected subcommand's `execute` method encounters an error.
    /// - There are issues with file operations or config loading/saving.
    ///
    pub async fn execute(
        self,
        config: TomlConfig,
        fs: &impl FileSystem,
        loader: &impl ConfigLoader,
        git: Arc<dyn Git>,
    ) -> Result<()> {
        match self {
            Self::Config(cmd) => cmd.execute(config, fs, loader, git).await,
        }
    }
}
