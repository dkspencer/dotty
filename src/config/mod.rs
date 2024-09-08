// Standard library imports
use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::PathBuf,
    str::FromStr,
};

// External crate imports
use anyhow::{Context, Error, Result};
use clients::file_system::FileSystem;
use colored::Colorize;
use log::{self, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use serde::{Deserialize, Serialize};
use toml;

// Submodules
pub mod command;
pub mod wizard;

pub type ProfileId = String;
pub type ProfilesMap = BTreeMap<ProfileId, ProfileConfig>;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Clone)]
pub struct ProfileConfig {
    pub branch: String,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            branch: String::from("main"),
        }
    }
}

pub trait ConfigLoader {
    fn get_base_path(&self) -> Result<PathBuf>;
    fn config_from_str(&self, content: &str) -> Result<TomlConfig>;
    fn config_to_string(&self, config: &TomlConfig) -> Result<String>;
}

pub struct ConfigLoaderClient;

impl ConfigLoader for ConfigLoaderClient {
    /// Determines and creates the base path for Dotty's configuration.
    ///
    /// This function decides the appropriate base path for Dotty's configuration files
    /// based on whether the application is running under Cargo (development mode) or not.
    ///
    /// After determining the path, this function attempts to create all necessary
    /// directories in the path if they don't already exist.
    ///
    /// # Returns
    /// Returns a `Result<PathBuf>` where:
    /// - `Ok(PathBuf)` contains the created or existing base path for Dotty's configuration.
    /// - `Err` is returned if there's an error accessing necessary directories or creating the path.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - It cannot access the current directory (when running under Cargo).
    /// - It cannot access the home directory (in production mode).
    /// - It fails to create the necessary directories.
    ///
    /// # Panics
    /// This function will panic if it fails to create the required directories.
    ///
    fn get_base_path(&self) -> Result<PathBuf> {
        let path = match Self::is_running_under_cargo() {
            true => env::current_dir()
                .context("Unable to access the current directory.")?
                .join(".config/dotty"),
            _ => home::home_dir()
                .context("Unable to access the home directory.")?
                .join(".config/dotty"),
        };

        fs::create_dir_all(&path).expect("Unable to create directories required by Dotty.");

        Ok(path)
    }

    /// Parses a TOML configuration string into a `TomlConfig` struct.
    ///
    /// This function takes a string containing TOML-formatted configuration data
    /// and attempts to deserialize it into a `TomlConfig` struct.
    ///
    /// # Arguments
    /// * `content` - A string slice containing the TOML configuration data.
    ///
    /// # Returns
    /// Returns a `Result` which is:
    /// * `Ok(TomlConfig)` if the parsing was successful.
    /// * `Err(Error)` if there was an error during parsing or deserialization.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The TOML content is malformed or cannot be parsed.
    /// * The parsed TOML does not match the structure of `TomlConfig`.
    ///
    fn config_from_str(&self, content: &str) -> Result<TomlConfig> {
        toml::from_str(content).map_err(Error::from)
    }

    /// Serializes a `TomlConfig` struct into a TOML-formatted string.
    ///
    /// This function takes a reference to a `TomlConfig` struct and attempts to
    /// serialize it into a TOML-formatted string representation.
    ///
    /// # Arguments
    /// * `config` - A reference to the `TomlConfig` struct to be serialized.
    ///
    /// # Returns
    /// Returns a `Result` which is:
    /// * `Ok(String)` containing the TOML-formatted string if serialization was successful.
    /// * `Err(Error)` if there was an error during serialization.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The `TomlConfig` struct contains data that cannot be represented in TOML format.
    /// * There's an internal error in the TOML serialization process.
    ///
    fn config_to_string(&self, config: &TomlConfig) -> Result<String> {
        toml::to_string(config).map_err(Error::from)
    }
}

impl ConfigLoaderClient {
    pub fn is_running_under_cargo() -> bool {
        env::var("CARGO").is_ok()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomlConfig {
    pub base_path: PathBuf,
    pub log_level: LevelFilter,
    pub profiles: ProfilesMap,
    pub active_profile: ProfileId,
}

impl Default for TomlConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            log_level: LevelFilter::Warn,
            profiles: BTreeMap::new(),
            active_profile: String::new(),
        }
    }
}

impl TomlConfig {
    fn default_with_base_path(base_path: PathBuf) -> Self {
        TomlConfig {
            base_path,
            ..TomlConfig::default()
        }
    }

    /// Loads the configuration from the TOML file or creates a default configuration
    /// if the TOML file does not exist.
    ///
    /// This function attempts to read, and parse, the TOML configuration file located at
    /// `<base_path>/config.toml`.
    ///
    /// If the file exists, it is read and parsed.
    /// If the file doesn't exist, we create (and save) a default configuration.
    ///
    /// # Arguments
    /// * `fs` - An implementation of the `FileSystem` trait used for file operations.
    /// * `loader` - An implementation of the `ConfigLoader` trait used to get the base
    ///              path and parse the configuration.
    ///
    /// # Returns
    /// Returns a `Result<Self>` where:
    /// - `Ok(Self)` contains the loaded or default configuration.
    /// - `Err` is returned if there's an error reading, parsing, or writing the configuration file.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - There's an issue accessing or reading the existing configuration file.
    /// - The existing configuration file cannot be parsed as valid TOML.
    /// - There's an error writing the default configuration to the file system.
    ///
    pub async fn from_path_or_default(
        fs: &impl FileSystem,
        loader: &impl ConfigLoader,
    ) -> Result<Self> {
        let path = loader.get_base_path()?.join("config.toml");

        if fs.exists(&path) {
            match fs.read_to_string(&path).await {
                Ok(content) => match loader.config_from_str(&content) {
                    Ok(config) => Ok(config),
                    Err(error) => {
                        log::error!("Error parsing config: {} :: {}", path.display(), error);
                        anyhow::bail!(
                            "Unable to parse existing config file at: {}",
                            path.display()
                        )
                    }
                },
                Err(error) => {
                    log::error!("Error reading config: {} :: {}", path.display(), error);
                    anyhow::bail!(
                        "Unable to reading existing config file at: {}",
                        path.display()
                    )
                }
            }
        } else {
            let config = Self::default_with_base_path(loader.get_base_path()?);
            fs.write(&path, &toml::to_string(&config)?).await?;
            Ok(config)
        }
    }

    /// Configures the logging system for Dotty based on the current configuration and runtime environment.
    ///
    /// This function sets up logging using log4rs, creating a file appender that writes to 'dotty.log'
    /// in the base path directory. The logging format and level are determined by the configuration
    /// and whether the application is running in development mode.
    ///
    /// # Arguments
    /// * `is_running_under_cargo` - A function that returns a boolean indicating whether
    ///   the application is running under Cargo (development mode).
    ///
    /// # Returns
    /// Returns a `Result<()>` where:
    /// - `Ok(())` indicates successful configuration of the logging system.
    /// - `Err` is returned if there's an error during the logging setup process.
    ///
    /// # Errors
    /// This function may return an error if:
    /// - There's an issue creating the log file or its parent directories.
    /// - The specified log level in the configuration is invalid.
    /// - There's a failure in initializing the log4rs configuration.
    ///
    pub async fn configure_logging(&self, is_running_under_cargo: fn() -> bool) -> Result<()> {
        let pattern = if is_running_under_cargo() {
            println!("{}", "Dotty is running in development mode.".red().bold());
            PatternEncoder::default()
        } else {
            PatternEncoder::new("{d} {l} - {m}\n")
        };
        let logfile = FileAppender::builder()
            .encoder(Box::new(pattern))
            .build(self.base_path.join("dotty.log"))?;

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(LevelFilter::from_str(self.log_level.as_str())?),
            )?;

        log4rs::init_config(config)?;

        Ok(())
    }

    pub async fn get_profile_ids(&self) -> HashSet<String> {
        self.profiles.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mockall::{mock, predicate::*};
    use std::{path::Path, sync::Arc};

    mod test_from_path_or_default {
        use super::*;

        mock! {
            FileSystem {}
            #[async_trait]
            impl FileSystem for FileSystem {
                fn exists(&self, path: &Path) -> bool;
                async fn read_to_string(&self, path: &Path) -> Result<String>;
                async fn write(&self, path: &Path, contents: &str) -> Result<()>;
            }
        }

        mock! {
            ConfigLoader {}
            impl ConfigLoader for ConfigLoader {
                fn get_base_path(&self) -> Result<PathBuf>;
                fn config_from_str(&self, content: &str) -> Result<TomlConfig>;
                fn config_to_string(&self, config: &TomlConfig) -> Result<String>;
            }
        }

        // Helper function for common setup
        fn setup_mocks() -> (MockFileSystem, MockConfigLoader, Arc<PathBuf>, PathBuf) {
            let mock_fs = MockFileSystem::new();
            let mut mock_loader = MockConfigLoader::new();
            let base_path = Arc::new(PathBuf::from("/test"));
            let config_path = base_path.join("config.toml");

            let base_path_clone = Arc::clone(&base_path);
            mock_loader
                .expect_get_base_path()
                .returning(move || Ok((*base_path_clone).clone()));

            (mock_fs, mock_loader, base_path, config_path)
        }

        #[tokio::test]
        async fn test_from_path_or_default_existing_file_is_valid() {
            let (mut mock_fs, mut mock_loader, base_path, config_path) = setup_mocks();
            let base_path_clone = Arc::clone(&base_path);

            // Set up expectations
            mock_loader
                .expect_get_base_path()
                .returning(move || Ok((*base_path_clone).clone()));

            // Test case 1: Config file exists and is valid
            mock_fs
                .expect_exists()
                .with(eq(config_path.clone()))
                .return_const(true);
            mock_fs
                .expect_read_to_string()
                .with(eq(config_path.clone()))
                .returning(|_| Ok("valid_config_content".to_string()));
            mock_loader
                .expect_config_from_str()
                .with(eq("valid_config_content"))
                .returning(|_| Ok(TomlConfig::default()));

            let result = TomlConfig::from_path_or_default(&mock_fs, &mock_loader).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_from_path_or_default_existing_file_is_invalid() {
            let (mut mock_fs, mut mock_loader, base_path, config_path) = setup_mocks();
            let base_path_clone = Arc::clone(&base_path);

            // Set up expectations
            mock_loader
                .expect_get_base_path()
                .returning(move || Ok((*base_path_clone).clone()));

            // Test case 2: Config file exists but is invalid
            mock_fs
                .expect_exists()
                .with(eq(config_path.clone()))
                .return_const(true);
            mock_fs
                .expect_read_to_string()
                .with(eq(config_path.clone()))
                .returning(|_| Ok("invalid_config_content".to_string()));
            mock_loader
                .expect_config_from_str()
                .with(eq("invalid_config_content"))
                .returning(|_| Err(anyhow::anyhow!("Invalid config")));

            let result = TomlConfig::from_path_or_default(&mock_fs, &mock_loader).await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_from_path_or_default_new_file() {
            let (mut mock_fs, mut mock_loader, base_path, config_path) = setup_mocks();
            let base_path_clone = Arc::clone(&base_path);

            // Set up expectations
            mock_loader
                .expect_get_base_path()
                .returning(move || Ok((*base_path_clone).clone()));

            // Test case 3: Config file doesn't exist, create default
            mock_fs
                .expect_exists()
                .with(eq(config_path.clone()))
                .return_const(false);

            mock_fs
                .expect_write()
                .with(eq(config_path), always())
                .returning(|_, _| Ok(()));

            let result = TomlConfig::from_path_or_default(&mock_fs, &mock_loader).await;
            assert!(result.is_ok());
        }
    }

    mod test_configure_logging {
        use super::*;
        use log::{max_level, LevelFilter};
        use tempfile::TempDir;

        fn setup_mocks() -> (TomlConfig, TempDir) {
            // Create a temporary directory for testing
            let temp_dir = TempDir::new().unwrap();
            let base_path = temp_dir.path().to_path_buf();

            // Create a mock TomlConfig
            let config = TomlConfig {
                base_path,
                log_level: LevelFilter::Info,
                profiles: BTreeMap::new(),
                active_profile: String::new(),
            };

            (config, temp_dir)
        }

        fn teardown_mocks(temp_dir: TempDir) {
            // Clean up the temporary directory
            temp_dir.close().unwrap();
        }

        #[tokio::test]
        async fn test_configure_logging() {
            let (config, temp_dir) = setup_mocks();

            // Mock the is_running_under_cargo function
            let is_running_under_cargo = || false;

            // Call the configure_logging function
            let result = config.configure_logging(is_running_under_cargo).await;

            // Assert that the function executed without errors
            assert!(result.is_ok());

            // Check if the log file was created
            let log_file_path = config.base_path.join("dotty.log");
            assert!(log_file_path.exists());

            // Verify that the logging is configured correctly
            assert_eq!(max_level(), LevelFilter::Info);

            teardown_mocks(temp_dir);
        }
    }
}
