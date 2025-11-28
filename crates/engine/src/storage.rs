//! Storage management for torrent data
//!
//! Handles file allocation and persistence

use std::path::PathBuf;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};

/// Manages file storage for a torrent
pub struct Storage {
    /// Base directory for downloads
    base_path: PathBuf,
}

impl Storage {
    /// Create a new storage manager
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the base download path
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Ensure a directory exists
    pub async fn ensure_dir(&self, relative_path: &str) -> std::io::Result<PathBuf> {
        let full_path = self.base_path.join(relative_path);
        fs::create_dir_all(&full_path).await?;
        Ok(full_path)
    }

    /// Pre-allocate space for a file
    pub async fn allocate_file(&self, relative_path: &str, size: u64) -> std::io::Result<()> {
        let full_path = self.base_path.join(relative_path);
        
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&full_path)
            .await?;

        file.set_len(size).await?;
        Ok(())
    }

    /// Write data to a specific position in a file
    pub async fn write_at(
        &self,
        relative_path: &str,
        offset: u64,
        data: &[u8],
    ) -> std::io::Result<()> {
        let full_path = self.base_path.join(relative_path);
        
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&full_path)
            .await?;

        file.seek(SeekFrom::Start(offset)).await?;
        file.write_all(data).await?;
        file.flush().await?;

        Ok(())
    }

    /// Read data from a specific position in a file
    pub async fn read_at(
        &self,
        relative_path: &str,
        offset: u64,
        length: usize,
    ) -> std::io::Result<Vec<u8>> {
        let full_path = self.base_path.join(relative_path);
        
        let mut file = File::open(&full_path).await?;
        file.seek(SeekFrom::Start(offset)).await?;

        let mut buffer = vec![0u8; length];
        file.read_exact(&mut buffer).await?;

        Ok(buffer)
    }

    /// Check if a file exists
    pub async fn file_exists(&self, relative_path: &str) -> bool {
        self.base_path.join(relative_path).exists()
    }

    /// Get file size
    pub async fn file_size(&self, relative_path: &str) -> std::io::Result<u64> {
        let metadata = fs::metadata(self.base_path.join(relative_path)).await?;
        Ok(metadata.len())
    }

    /// Delete a file
    pub async fn delete_file(&self, relative_path: &str) -> std::io::Result<()> {
        let full_path = self.base_path.join(relative_path);
        if full_path.exists() {
            fs::remove_file(full_path).await?;
        }
        Ok(())
    }

    /// Delete a directory and its contents
    pub async fn delete_dir(&self, relative_path: &str) -> std::io::Result<()> {
        let full_path = self.base_path.join(relative_path);
        if full_path.exists() {
            fs::remove_dir_all(full_path).await?;
        }
        Ok(())
    }
}
