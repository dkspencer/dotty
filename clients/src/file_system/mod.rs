// Standard library imports
use std::path::Path;

// External crate imports
use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;

#[async_trait]
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    async fn read_to_string(&self, path: &Path) -> Result<String>;
    async fn write(&self, path: &Path, contents: &str) -> Result<()>;
}

pub struct FileSystemClient;

#[async_trait]
impl FileSystem for FileSystemClient {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    async fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path).await?)
    }

    async fn write(&self, path: &Path, contents: &str) -> Result<()> {
        fs::create_dir_all(path.parent().unwrap_or(path)).await?;
        fs::write(path, contents).await?;
        Ok(())
    }
}
