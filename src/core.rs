use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const FILE_PATH: &str = "data/database.bin";

pub struct Database {
    data: HashMap<String, u64>,
    file: File,
}

#[derive(Debug)]
pub enum DatabaseError {
    IoError(std::io::Error),
    SerializationError(Box<dyn std::error::Error>),
    DeserializationError(Box<dyn std::error::Error>),
}

impl std::error::Error for DatabaseError {}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::IoError(err) => write!(f, "IO error: {}", err),
            DatabaseError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            DatabaseError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl Database {
    pub fn new() -> Result<Self, DatabaseError> {
        let path = Path::new(FILE_PATH);
        if !path.exists() {
            let file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(path)?;
            // Initialize an empty database if the file doesn't exist
            let db = Database {
                data: HashMap::new(),
                file,
            };
            db.save_data()?;
        }

        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut db = Database {
            data: HashMap::new(),
            file,
        };

        db.load_data()?;
        Ok(db)
    }

    fn load_data(&mut self) -> Result<(), DatabaseError> {
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;

        let mut offset = 0;
        while offset < buffer.len() {
            let key_len = buffer[offset] as usize;
            offset += 1;
            let key = String::from_utf8_lossy(&buffer[offset..offset + key_len]).to_string();
            offset += key_len;

            let value = u64::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);
            offset += 8;

            self.data.insert(key, value);
        }

        Ok(())
    }

    fn save_data(&mut self) -> Result<(), DatabaseError> {
        let mut buffer = Vec::new();
        for (key, value) in &self.data {
            buffer.push(key.len() as u8);
            buffer.extend_from_slice(key.as_bytes());
            buffer.extend_from_slice(&value.to_le_bytes());
        }

        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&buffer)?;

        Ok(())
    }

    pub fn set(&mut self, key: String, value: u64) -> Result<(), DatabaseError> {
        self.data.insert(key.clone(), value);
        self.save_data()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&u64> {
        self.data.get(key)
    }
}