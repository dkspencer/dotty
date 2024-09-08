// Standard library imports
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use clap::{Parser, ValueEnum};
use cliclack;
use clients::{file_system::FileSystem, git::Git};
use crossterm::style::{style, Stylize};

// Local module imports
use crate::config::{
    wizard::{
        list_profiles_wizard, new_profile_wizard, select_profiles_wizard, update_profile_wizard,
    },
    ConfigLoader, TomlConfig,
};

#[derive(ValueEnum, Default, Debug, Clone)]
pub enum Command {
    /// List all profiles and select a profile to activate.
    #[default]
    List,

    /// Create a new profile.
    Create,

    /// Delete one or more profiles.
    Delete,

    /// Update an existing profile.
    Update,
}

/// Set up and manage existing Dotty Profiles through interactive wizards.
#[derive(Parser, Debug)]
pub struct ProfileCommand {
    #[clap(default_value_t, value_enum)]
    command: Command,
    // #[arg(long, default_value_t)]
}

impl ProfileCommand {
    pub async fn execute(
        self,
        mut config: TomlConfig,
        fs: &impl FileSystem,
        loader: &impl ConfigLoader,
        git: Arc<dyn Git>,
    ) -> Result<()> {
        match self.command {
            Command::List => {
                let current_profile = config.active_profile.clone();

                let config = list_profiles_wizard(config).await?;
                let contents = loader.config_to_string(&config)?;

                if config.active_profile != current_profile {
                    fs.write(&config.base_path.join("config.toml"), &contents)
                        .await?;

                    cliclack::outro(
                        style(format!(
                            "Active profile has been changed to: {}",
                            config.active_profile
                        ))
                        .green()
                        .bold(),
                    )?;
                }
            }
            Command::Create => {
                let config = new_profile_wizard(config, git).await?;
                let contents = loader.config_to_string(&config)?;

                fs.write(&config.base_path.join("config.toml"), &contents)
                    .await?;

                cliclack::outro(style("A new profile has been created!").green().bold())?;
            }
            Command::Delete => {
                let profiles = select_profiles_wizard(&config).await?;
                for profile in &profiles {
                    config.profiles.remove(profile);
                }

                let contents = loader.config_to_string(&config)?;

                fs.write(&config.base_path.join("config.toml"), &contents)
                    .await?;

                cliclack::outro(
                    style(format!("{} profile(s) have been deleted", profiles.len()))
                        .green()
                        .bold(),
                )?;
            }
            Command::Update => {
                let config = update_profile_wizard(config, git).await?;

                let contents = loader.config_to_string(&config)?;

                fs.write(&config.base_path.join("config.toml"), &contents)
                    .await?;

                cliclack::outro(style("Profile have been updated").green().bold())?;
            }
        }

        Ok(())
    }
}
