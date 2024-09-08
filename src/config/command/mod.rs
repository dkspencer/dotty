// Standard library imports
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use clap::Subcommand;

// Local module imports
use crate::{
    clients::{file_system::FileSystem, git::Git},
    config::{
        command::{profile::ProfileCommand, setup::SetupCommand},
        ConfigLoader, TomlConfig,
    },
};

// Submodules
pub mod profile;
pub mod setup;

/// Configure Dotty system settings and profiles.
#[derive(Debug, Subcommand)]
pub enum Commands {
    Setup(SetupCommand),
    Profile(ProfileCommand),
}

impl Commands {
    pub async fn execute(
        self,
        config: TomlConfig,
        fs: &impl FileSystem,
        loader: &impl ConfigLoader,
        git: Arc<dyn Git>,
    ) -> Result<()> {
        match self {
            Self::Setup(cmd) => cmd.execute(config, fs, loader, git).await,
            Self::Profile(cmd) => cmd.execute(config, fs, loader, git).await,
        }
    }
}
