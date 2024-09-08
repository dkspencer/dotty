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
            Err(anyhow!(
                "This name is already used. Please choose a different one."
            ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_branch_unique() {
        let git_client = GitClient;
        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "feature/123".to_string(),
        ];

        assert!(git_client
            .is_branch_unique(branches.clone(), "new-branch")
            .is_ok());
        assert!(git_client
            .is_branch_unique(branches.clone(), "main")
            .is_err());
        assert!(git_client.is_branch_unique(branches, "develop").is_err());
    }

    #[test]
    fn test_is_valid_branch_name() {
        let git_client = GitClient;

        // Valid branch names
        assert!(git_client.is_valid_branch_name("feature/123").is_ok());
        assert!(git_client.is_valid_branch_name("hotfix-456").is_ok());
        assert!(git_client.is_valid_branch_name("release_1.0").is_ok());

        // Invalid branch names
        assert!(git_client.is_valid_branch_name("").is_err());
        assert!(git_client.is_valid_branch_name(" ").is_err());
        assert!(git_client
            .is_valid_branch_name("/start-with-slash")
            .is_err());
        assert!(git_client.is_valid_branch_name("end-with-slash/").is_err());
        assert!(git_client.is_valid_branch_name("double..dot").is_err());
        assert!(git_client.is_valid_branch_name("contains space").is_err());
        assert!(git_client.is_valid_branch_name("invalid*char").is_err());
        assert!(git_client.is_valid_branch_name("invalid?char").is_err());
        assert!(git_client.is_valid_branch_name("invalid:char").is_err());
        assert!(git_client.is_valid_branch_name("invalid[char").is_err());
        assert!(git_client.is_valid_branch_name("invalid\\char").is_err());
        assert!(git_client.is_valid_branch_name("invalid^char").is_err());
        assert!(git_client.is_valid_branch_name("invalid~char").is_err());
    }
}
