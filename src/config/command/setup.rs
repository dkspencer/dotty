// Standard library imports
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use clap::Parser;

// Local module imports
use crate::{
    clients::{file_system::FileSystem, git::Git},
    config::{wizard::initial_setup_wizard, ConfigLoader, TomlConfig},
};

/// Sets up the application configuration through an interactive wizard.
#[derive(Parser, Debug)]
pub struct SetupCommand {}

impl SetupCommand {
    /// Sets up the initial configuration for Dotty.
    ///
    /// This function runs a configuration wizard, saves the resulting configuration
    /// to a TOML file, and writes it to the specified location.
    ///
    /// # Arguments
    /// * `self` - The SetupCommand instance.
    /// * `config` - The initial TomlConfig to be modified by the wizard.
    /// * `fs` - An implementation of FileSystem for file operations.
    /// * `loader` - An implementation of ConfigLoader for serializing the config.
    /// * `git` - An implementation of Git for interacting with the Git API.
    ///
    /// # Returns
    /// Returns `Ok(())` if the setup process completes successfully, or an error if
    /// any step fails.
    ///
    /// # Errors
    /// This function may return an error if:
    /// - The wizard function fails
    /// - The config serialization fails
    /// - Writing the config file fails
    ///
    pub async fn execute(
        self,
        config: TomlConfig,
        fs: &impl FileSystem,
        loader: &impl ConfigLoader,
        git: Arc<dyn Git>,
    ) -> Result<()> {
        let config = initial_setup_wizard(config, git).await?;
        let contents = loader.config_to_string(&config)?;

        fs.write(&config.base_path.join("config.toml"), &contents)
            .await?;
        Ok(())
    }
}
