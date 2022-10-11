use anyhow::{Context, Error, Result};
use std::fs::{self, File};
use std::io::SeekFrom;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct FileMetadata {
    timestamp: Duration,
    ttl_secs: Duration,
    pub content_type: Option<String>,
    pub content_length: u64,
}

impl FileMetadata {
    pub fn new(
        ttl: u64,
        content_length: u64,
        content_type: Option<String>,
    ) -> Result<FileMetadata> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context(format!("Failed to get system time"))?;
        Ok(FileMetadata {
            timestamp,
            ttl_secs: Duration::from_secs(ttl),
            content_length,
            content_type,
        })
    }

    fn from_buffer(buffer: &[u8], content_type: String) -> FileMetadata {
        assert!(buffer.len() == std::mem::size_of::<u64>() * 3);

        let (first, rest) = buffer.split_at(std::mem::size_of::<u64>());
        let (second, third) = rest.split_at(std::mem::size_of::<u64>());
        let timestamp = Duration::from_secs(u64::from_le_bytes(first.try_into().unwrap()));
        let ttl_secs = Duration::from_secs(u64::from_le_bytes(second.try_into().unwrap()));
        let content_length = u64::from_le_bytes(third.try_into().unwrap());
        let content_type = if content_type == "" {
            None
        } else {
            Some(content_type.trim().to_string())
        };

        FileMetadata {
            content_type,
            timestamp,
            ttl_secs,
            content_length,
        }
    }
}

impl FileMetadata {
    pub fn is_expired(&self) -> bool {
        if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
            return now >= self.timestamp + self.ttl_secs;
        } else {
            return false;
        }
    }

    fn size_bytes(&self) -> u64 {
        let a = if let Some(ct) = &self.content_type {
            ct.as_bytes().len() + 1
        } else {
            1
        };
        let b = std::mem::size_of::<u64>() * 3;

        return (a + b) as u64;
    }
}

pub struct CacheFile {
    pub metadata: FileMetadata,
    pub path: PathBuf,
    pub content_data: Vec<u8>,
}

impl CacheFile {
    pub fn new(
        ttl: u64,
        content_length: u64,
        path: PathBuf,
        content_data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<CacheFile> {
        if path.as_path().is_dir() {
            return Err(Error::msg("Cache path is directory"));
        } else {
            let metadata = FileMetadata::new(ttl, content_length, content_type)?;
            return Ok(CacheFile {
                metadata,
                path,
                content_data,
            });
        }
    }

    pub fn read_header(path: &PathBuf) -> Result<FileMetadata> {
        let file =
            File::open(path.as_path()).context(format!("Failed to read file header {path:?}"))?;

        let mut reader = BufReader::new(file);
        let mut content_type = String::new();
        let mut buffer = [0u8; std::mem::size_of::<u64>() * 3];

        reader
            .read_line(&mut content_type)
            .context(format!("Failed to read header from {path:?}"))?;
        reader
            .read_exact(&mut buffer)
            .context(format!("Failed to read header from {path:?}"))?;

        Ok(FileMetadata::from_buffer(&buffer, content_type))
    }

    pub fn read(path: PathBuf, metadata: FileMetadata) -> Result<CacheFile> {
        let file = File::open(path.as_path()).context(format!("Failed to read file {path:?}"))?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(metadata.size_bytes())).unwrap();

        let mut content_data: Vec<u8> = Vec::with_capacity(metadata.content_length as usize);
        if let Ok(_) = reader.read_to_end(&mut content_data) {
            Ok(CacheFile {
                metadata,
                path,
                content_data,
            })
        } else {
            Err(Error::msg(format!("Failed to read cache file")))
        }
    }

    pub fn write(&self) -> Result<()> {
        let ext = Uuid::new_v4();
        let mut path_tmp = self.path.clone();
        path_tmp.set_extension(ext.to_string());

        let parent = path_tmp
            .parent()
            .context(format!("Failed to get parent dir"))?;

        fs::create_dir_all(parent).context(format!("Failed to create parent dir {parent:?}"))?;

        let mut file =
            File::create(&path_tmp).context(format!("Failed to create file {path_tmp:?}"))?;
        let ctype_buff = if let Some(ct) = &self.metadata.content_type {
            let mut ct = ct.clone().trim().to_string();
            ct.push('\n');
            ct.as_bytes().to_vec()
        } else {
            let mut ct = String::new();
            ct.push('\n');
            ct.as_bytes().to_vec()
        };
        let timestamp_buff = self.metadata.timestamp.as_secs().to_le_bytes().to_vec();
        let ttl_buff = self.metadata.ttl_secs.as_secs().to_le_bytes().to_vec();
        let len_buff = self.metadata.content_length.to_le_bytes().to_vec();

        // Write header
        let header_buff = [ctype_buff, timestamp_buff, ttl_buff, len_buff].concat();
        let mut pos = 0;
        while pos < header_buff.len() {
            if let Ok(bytes_written) = file.write(&header_buff[pos..]) {
                pos += bytes_written;
                file.flush().unwrap();
            } else {
                return Err(Error::msg(format!(
                    "Failed to write file header {path_tmp:?}"
                )));
            }
        }

        // Write data
        let data_buff = self.content_data.as_slice();
        pos = 0;

        while pos < data_buff.len() {
            if let Ok(bytes_written) = file.write(&data_buff[pos..]) {
                pos += bytes_written;
                file.flush().unwrap();
            } else {
                return Err(Error::msg(format!("Failed to write file {path_tmp:?}")));
            }
        }

        std::fs::rename(&path_tmp.as_path(), &self.path.as_path()).unwrap();

        Ok(())
    }
}

pub fn mk_file_path(cache_dir: &PathBuf, uri: String) -> PathBuf {
    let mut path = cache_dir.clone();

    if uri.starts_with("/") {
        path.push(uri.strip_prefix("/").unwrap());
    } else {
        path.push(uri);
    }

    path
}

pub fn delete_cache_file(path: PathBuf) -> Result<()> {
    fs::remove_file(path.as_path()).context(format!("Failed to delete file at {path:?}"))
}
