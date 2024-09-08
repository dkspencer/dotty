// Standard library imports
use std::{collections::HashSet, sync::Arc};

// External crate imports
use anyhow::Result;
use cliclack;
use crossterm::style::{style, Stylize};
use log::LevelFilter;

// Local module imports
use crate::{
    clients::git::Git,
    config::{ProfileConfig, ProfileId, TomlConfig},
};

/// Guides the user through the initial setup of Dotty.
///
/// This function is executed when Dotty is launched for the first time or when
/// a fresh configuration is needed. It walks the user through setting up system-wide
/// settings and creating at least one profile.
///
/// # Parameters
/// * `config`: A `TomlConfig` struct, typically starting with default values.
/// * `git` - An implementation of Git for interacting with the Git API.
///
/// # Returns
/// * `Result<TomlConfig>`: The fully configured `TomlConfig` if successful, or an error
///   if any part of the setup process fails.
///
/// # Process
/// 1. Configures system-wide settings by calling `system_settings`.
/// 2. If no profiles exist, guides the user through creating an initial profile
///    by calling `new_profile_wizard`.
///
/// # Errors
/// This function may return an error if:
/// * The screen cannot be cleared.
/// * The system settings configuration fails.
/// * The profile creation process fails (when needed).
/// * Any I/O or user interaction errors occur during the setup process.
///
pub async fn initial_setup_wizard(mut config: TomlConfig, git: Arc<dyn Git>) -> Result<TomlConfig> {
    cliclack::clear_screen()?;
    cliclack::intro(style(" Configure Dotty ").on_dark_green().black().bold())?;

    config = system_settings(config).await?;

    if config.profiles.is_empty() {
        cliclack::log::info(
            "A Profile is like a container for a specific look and feel of your \
            system. You can only use one Profile at a time. When you use Dotty, \
            it will apply the settings from your currently active Profile.",
        )?;

        if !cliclack::confirm("Do you want to create a profile?").interact()? {
            return Ok(config);
        };

        config = new_profile_wizard(config, git).await?;
    }

    Ok(config)
}

/// Guides the user through configuring system-wide settings for Dotty.
///
/// This function presents an interactive wizard to the user, allowing them to set
/// critical configuration options such as; the base path for storing Dotty files
/// and the desired level of detail for activity reports (log level).
///
/// # Parameters
/// * `config`: A `TomlConfig` struct containing the current configuration settings.
///
/// # Returns
/// * `Result<TomlConfig>`: The updated configuration if successful, or an error if
///                         the user interaction fails or invalid input is provided.
///
/// # Errors
/// This function may return an error if:
/// * User input cannot be read.
/// * The provided base path is invalid
///
async fn system_settings(mut config: TomlConfig) -> Result<TomlConfig> {
    cliclack::log::info(
        "Welcome! We'll guide you through setting up Dotty step by step. \
        We'll ask you a few simple questions to help Dotty work best for you. \
        This will make sure Dotty has all the information it needs to run smoothly on your computer.",
    )?;

    config.base_path = cliclack::input(
        style("Enter the full folder path where you want Dotty to save its files").bold(),
    )
    .default_input(config.base_path.to_string_lossy().as_ref())
    .validate(|input: &String| {
        if !input.contains(std::path::is_separator) {
            Err("Please enter an absolute path")
        } else {
            Ok(())
        }
    })
    .interact()?;

    config.log_level =
        cliclack::select(style("How much detail do you want in Dotty's activity reports?").bold())
            .initial_value(LevelFilter::Warn)
            .items(&[
                (LevelFilter::Off, "Off", "Disable logging"),
                (LevelFilter::Debug, "Debug", "Show all possible details"),
                (
                    LevelFilter::Info,
                    "Info",
                    "Show general updates and information",
                ),
                (
                    LevelFilter::Warn,
                    "Warn",
                    "Show potential issues and concerns",
                ),
                (
                    LevelFilter::Error,
                    "Error",
                    "Show serious problems that need attention",
                ),
            ])
            .interact()?;

    Ok(config)
}

/// Guides the user through creating a new profile for Dotty.
///
/// This function presents an interactive wizard to the user set up a new profile.
///
/// # Parameters
/// * `config`: A mutable `TomlConfig` struct containing the current Dotty configuration.
/// * `git` - An implementation of Git for interacting with the Git API.
///
/// # Returns
/// * `Result<TomlConfig>`: The updated configuration if successful, or an error if
///                         the user interaction fails or invalid input is provided.
///
///
/// # Errors
/// This function may return an error if:
/// * The screen cannot be cleared.
/// * User input cannot be read.
/// * The provided profile ID or Git branch name is not unique.
///
pub async fn new_profile_wizard(mut config: TomlConfig, git: Arc<dyn Git>) -> Result<TomlConfig> {
    cliclack::clear_screen()?;

    cliclack::intro(style(" Create a Dotty Profile ").on_dark_green().bold())?;

    let existing_ids: HashSet<_> = config.profiles.keys().cloned().collect();
    let profile_id: ProfileId =
        cliclack::input(style("Assign a unique identifier for this profile.").bold())
            .placeholder("nord-theme")
            .validate(move |input: &String| {
                if existing_ids.contains(input) {
                    Err("Profile with this ID already exists.")
                } else {
                    Ok(())
                }
            })
            .interact()?;

    cliclack::log::info(
        "Dotty works with your Git account to keep track of your settings. Think of Git as a \
        storage system that remembers different versions of your settings. \
        For each Profile you create in Dotty, we'll use a separate 'branch' in Git. \
        This is like having a separate folder for each set of settings, keeping everything organized.",
    )?;

    // Collect now because `set_profile` is used elsewhere.
    let branches: Vec<String> = config
        .profiles
        .values()
        .map(|profile| profile.branch.to_string())
        .collect();

    let profile = set_profile(None, git, branches).await?;

    config.profiles.insert(profile_id.clone(), profile);

    if cliclack::confirm(
        style("Would you like to set this profile as your main choice for Dotty?").bold(),
    )
    .interact()?
    {
        config.active_profile = profile_id
    }

    Ok(config)
}

/// Configures and sets up a profile for the application.
///
/// This function prompts the user to input a unique name for the profile's storage space in Git
/// (referred to as a 'branch'). It validates the input against Git branch naming rules and ensures
/// the branch name is unique among existing branches.
///
/// # Arguments
/// * `profile_` - An optional `ProfileConfig` to start with. If None, a default profile is created.
/// * `git` - An `Arc<dyn Git>` representing the Git interface for validation.
/// * `branches` - A vector of existing branch names to check for uniqueness.
///
/// # Returns
/// Returns a `Result<ProfileConfig>` containing the configured profile if successful,
/// or an error if the input validation fails or the user interaction is interrupted.
///
/// # Errors
/// This function can return an error if:
/// - The inputted branch name is invalid according to Git rules.
/// - The branch name is not unique among existing branches.
/// - There's an issue with the user interaction (e.g., CLI input/output errors).
async fn set_profile(
    profile_: Option<ProfileConfig>,
    git: Arc<dyn Git>,
    branches: Vec<String>,
) -> Result<ProfileConfig> {
    let mut profile = profile_.unwrap_or_default();

    profile.branch = cliclack::input(
        style(
            "Give a unique name for this Profile's storage space in Git (we call this a 'branch')",
        )
        .bold(),
    )
    .default_input(&profile.branch)
    .validate(move |input: &String| {
        if let Err(e) = git.is_valid_branch_name(input) {
            return Err(e.to_string());
        }

        if let Err(e) = git.is_branch_unique(branches.clone(), input) {
            return Err(e.to_string());
        }

        Ok(())
    })
    .interact()?;

    Ok(profile)
}

/// Presents a wizard for listing and selecting Dotty profiles.
///
/// This function displays all configured Dotty profiles to the user and allows
/// them to optionally select one as the active profile.
///
/// # Parameters
/// * `config`: A `TomlConfig` struct containing the current configuration settings.
///
/// # Returns
/// * `Result<TomlConfig>` - The updated configuration with potentially a new active profile,
///   wrapped in a Result. Returns an error if any CLI operations fail.
///
/// # Errors
/// This function may return an error if:
/// * The screen cannot be cleared.
/// * User input cannot be read.
///
pub async fn list_profiles_wizard(mut config: TomlConfig) -> Result<TomlConfig> {
    cliclack::clear_screen()?;
    cliclack::intro(style(" Dotty Profiles ").on_dark_green().bold())?;

    let profile_ids = config.get_profile_ids().await;

    let options: Vec<(String, String, String)> = profile_ids
        .into_iter()
        .map(|key| (key.clone(), key.clone(), String::new()))
        .collect();

    config.active_profile = cliclack::select(
        style("Select a profile to activate as your default Dotty profile.").bold(),
    )
    .initial_value(config.active_profile)
    .items(&options)
    .interact()?;

    Ok(config)
}

/// Presents a wizard interface for selecting multiple profiles to delete.
///
/// This function displays a clear screen with a styled intro, then presents the user
/// with a multi-select interface to choose one or more profiles for deletion.
///
/// # Arguments
/// * `config` - A reference to the `TomlConfig` containing the current configuration.
///
/// # Returns
/// Returns a `Result<Vec<ProfileId>>` containing the IDs of the selected profiles if successful,
/// or an error if the user interaction fails.
///
/// # Errors
/// This function can return an error if:
/// - There's an issue clearing the screen or displaying the intro.
/// - There's a problem retrieving the profile IDs from the config.
/// - The user interaction for multi-select fails (e.g., CLI input/output errors).
///
pub async fn select_profiles_wizard(config: &TomlConfig) -> Result<Vec<ProfileId>> {
    cliclack::clear_screen()?;
    cliclack::intro(style(" Dotty Profiles ").on_dark_green().black().bold())?;

    let profile_ids = config.get_profile_ids().await;

    let options: Vec<(String, String, String)> = profile_ids
        .into_iter()
        .map(|key| (key.clone(), key.clone(), String::new()))
        .collect();

    let profiles = cliclack::multiselect(style("Select one or more profiles to delete.").bold())
        .required(true)
        .items(&options)
        .interact()?;

    Ok(profiles)
}

/// Presents a wizard interface for updating an existing profile in the configuration.
///
/// This function displays a clear screen with a styled intro, then guides the user through
/// selecting a profile to update and modifying its details.
///
/// # Arguments
/// * `config` - The current `TomlConfig` containing all profiles.
/// * `git` - An `Arc<dyn Git>` representing the Git interface for validation during profile update.
///
/// # Returns
/// Returns a `Result<TomlConfig>` containing the updated configuration if successful,
/// or an error if any step of the process fails.
///
/// # Errors
/// This function can return an error if:
/// - There's an issue clearing the screen or displaying the intro.
/// - There's a problem retrieving the profile IDs from the config.
/// - The user interaction for selecting a profile fails.
/// - The selected profile is not found in the configuration.
/// - The profile update process (via `set_profile`) encounters an error.
///
pub async fn update_profile_wizard(
    mut config: TomlConfig,
    git: Arc<dyn Git>,
) -> Result<TomlConfig> {
    cliclack::clear_screen()?;
    cliclack::intro(style(" Dotty Profiles ").on_dark_green().black().bold())?;

    let profile_ids = config.get_profile_ids().await;

    let options: Vec<(String, String, String)> = profile_ids
        .into_iter()
        .map(|key| (key.clone(), key.clone(), String::new()))
        .collect();

    let selected_profile_id = cliclack::select(style("Select a profile to update.").bold())
        .items(&options)
        .interact()?;

    let selected_profile = config
        .profiles
        .get(&selected_profile_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No profile found with ID: {}", selected_profile_id))?;

    let branches: Vec<String> = config
        .profiles
        .values()
        .map(|profile| profile.branch.to_string())
        .filter(|branch| branch != &selected_profile.branch)
        .collect();

    let profile = set_profile(Some(selected_profile), git, branches).await?;

    config.profiles.insert(selected_profile_id, profile);

    Ok(config)
}
