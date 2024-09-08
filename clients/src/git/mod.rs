// External crate imports
use anyhow::{anyhow, Result};

pub trait Git {
    fn is_branch_unique(&self, branches: Vec<String>, name: &str) -> Result<()>;
    fn is_valid_branch_name(&self, name: &str) -> Result<()>;
}

pub struct GitClient;

impl Git for GitClient {
    fn is_branch_unique(&self, branches: Vec<String>, name: &str) -> Result<()> {
        if branches.iter().any(|branch| branch == name) {
            return Err(anyhow!(
                "This name is already used. Please choose a different one."
            ));
        } else {
            Ok(())
        }
    }
    fn is_valid_branch_name(&self, name: &str) -> Result<()> {
        const INVALID_CHARS: [char; 7] = ['~', '^', ':', '?', '*', '[', '\\'];

        if name.trim().is_empty() {
            return Err(anyhow!("Branch name cannot be empty"));
        }

        if name.starts_with('/') || name.ends_with('/') {
            return Err(anyhow!("Branch name cannot start or end with '/'"));
        }

        if name.contains("..") {
            return Err(anyhow!(
                "Branch name cannot contain two consecutive dots '..'"
            ));
        }

        name.chars().try_for_each(|c| {
            if c.is_whitespace() {
                Err(anyhow!("Branch name cannot contain spaces"))
            } else if INVALID_CHARS.contains(&c) || c.is_control() {
                Err(anyhow!("Branch name contains invalid characters"))
            } else {
                Ok(())
            }
        })?;

        Ok(())
    }
}
