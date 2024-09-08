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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;

    #[test]
    async fn test_exists() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let fs_client = FileSystemClient;

        assert!(!fs_client.exists(&file_path));

        tokio::fs::write(&file_path, "test content").await.unwrap();

        assert!(fs_client.exists(&file_path));
    }

    #[test]
    async fn test_read_to_string() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let content = "Hello, world!";

        tokio::fs::write(&file_path, content).await.unwrap();

        let fs_client = FileSystemClient;
        let result = fs_client.read_to_string(&file_path).await.unwrap();

        assert_eq!(result, content);
    }

    #[test]
    async fn test_write() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let content = "Test content";

        let fs_client = FileSystemClient;
        fs_client.write(&file_path, content).await.unwrap();

        let read_content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    async fn test_write_creates_directories() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("test_file.txt");
        let content = "Nested content";

        let fs_client = FileSystemClient;
        fs_client.write(&nested_path, content).await.unwrap();

        assert!(nested_path.exists());
        let read_content = tokio::fs::read_to_string(&nested_path).await.unwrap();
        assert_eq!(read_content, content);
    }
}
