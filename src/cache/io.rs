use anyhow::{Context, Error, Result};
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::SeekFrom;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct FileMetadata {
    timestamp: Duration,
    ttl_secs: Duration,
    content_length: u64,
}

impl FileMetadata {
    pub fn new(ttl: u64, content_length: u64) -> Result<FileMetadata> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context(format!("Failed to get system time"))?;
        Ok(FileMetadata {
            timestamp,
            ttl_secs: Duration::from_secs(ttl),
            content_length,
        })
    }

    fn from_buffer(buffer: &[u8]) -> Result<FileMetadata> {
        assert!(buffer.len() == 16);
        let (first, rest) = buffer.split_at(std::mem::size_of::<u64>());
        let (second, third) = rest.split_at(std::mem::size_of::<u64>());
        let timestamp = Duration::from_secs(u64::from_le_bytes(first.try_into().unwrap()));
        let ttl_secs = Duration::from_secs(u64::from_le_bytes(second.try_into().unwrap()));
        let content_length = u64::from_le_bytes(second.try_into().unwrap());

        Ok(FileMetadata {
            timestamp,
            ttl_secs,
            content_length,
        })
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
}

pub struct CacheFile {
    metadata: FileMetadata,
    path: PathBuf,
    content_data: Vec<u8>,
}

impl CacheFile {
    pub fn new(
        ttl: u64,
        content_length: u64,
        path: PathBuf,
        content_data: Vec<u8>,
    ) -> Result<CacheFile> {
        if path.as_path().is_dir() {
            return Err(Error::msg("Cache path is directory"));
        } else {
            let metadata = FileMetadata::new(ttl, content_length)?;
            return Ok(CacheFile {
                metadata,
                path,
                content_data,
            });
        }
    }

    pub fn read_header(path: PathBuf) -> Result<FileMetadata> {
        let mut file =
            File::open(path.as_path()).context(format!("Failed to read file header {path:?}"))?;
        let mut buffer = [0u8; std::mem::size_of::<u64>() * 2];
        file.read_exact(&mut buffer)
            .context(format!("Failed to read header from {path:?}"))?;

        FileMetadata::from_buffer(&buffer)
    }

    pub fn read(path: PathBuf, metadata: FileMetadata) -> Result<CacheFile> {
        let file = File::open(path.as_path()).context(format!("Failed to read file {path:?}"))?;
        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start((std::mem::size_of::<u64>() * 2) as u64))
            .unwrap();

        let data_size = metadata.content_length;
        let buff_size = if data_size < 2048 {
            data_size as usize
        } else {
            data_size as usize / 1024
        };
        let buffer = vec![0u8; buff_size];
        let mut buffer = buffer.clone();
        let buffer = RefCell::new(buffer.as_mut_slice());
        let mut content_data: Vec<u8> = Vec::new();
        let mut bytes_read = 0;

        while bytes_read < data_size {
            let mut pos = 0;
            while pos < buff_size {
                if let Ok(buff_bytes_read) = reader.read(&mut buffer.borrow_mut()[pos..]) {
                    pos += buff_bytes_read;
                } else {
                    return Err(Error::msg(format!("Failed to read file {path:?}")));
                }
            }

            bytes_read += pos as u64;
            buffer.borrow().iter().for_each(|b| content_data.push(*b));
        }

        Ok(CacheFile {
            metadata,
            path,
            content_data,
        })
    }

    pub fn write(&self) -> Result<()> {
        let path = self.path.as_path();
        let parent = path.parent().context(format!("Failed to get parent dir"))?;

        fs::create_dir_all(parent).context(format!("Failed to create parent dir {parent:?}"))?;

        let mut file = File::create(path).context(format!("Failed to create file {path:?}"))?;
        let timestamp_buff: [u8; 8] = self
            .metadata
            .timestamp
            .as_secs()
            .to_le_bytes()
            .try_into()
            .unwrap();
        let ttl_buff: [u8; 8] = self
            .metadata
            .ttl_secs
            .as_secs()
            .to_le_bytes()
            .try_into()
            .unwrap();

        let len_buff: [u8; 8] = self
            .metadata
            .content_length
            .to_le_bytes()
            .try_into()
            .unwrap();

        // Write header
        let header_buff = [timestamp_buff, ttl_buff, len_buff].concat();
        let mut pos = 0;
        while pos < header_buff.len() {
            if let Ok(bytes_written) = file.write(&header_buff[pos..]) {
                pos += bytes_written;
                file.flush().unwrap();
            } else {
                return Err(Error::msg(format!("Failed to write file header {path:?}")));
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
                return Err(Error::msg(format!("Failed to write file {path:?}")));
            }
        }

        Ok(())
    }

    pub fn delete(path: PathBuf) -> Result<()> {
        fs::remove_file(path.as_path()).context(format!("Failed to delete file at {path:?}"))
    }
}
