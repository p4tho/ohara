use serde::{ Serialize, Deserialize };
use serde_json;
use std::collections::HashMap;
use thiserror::Error;

/*
 * storage.rs:
 * Reading progress is stored on disk as a .json file that represents a hashmap.
 * The keys are SHA-256 hashes of raw .pdf bytes and the values are progress data (M4BData).
 * Each OS has a different way to retrieve files, so json will represent hashmap.
 * This leaves file operations to OS-specific apps. So all progress data is based on one .json file.
 */
 
type PDFHash = String;
 
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Storage {
    pub m4b_data: HashMap<PDFHash, M4BData>
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("serialization or deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct M4BData {
    pub path: String,           // where .m4b file is located
    pub cur_timestamp: u64,     // mesaures how many ms listened to so far
    pub last_updated: u64       // last time updated (unix)
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load(json_str: &str) -> Result<Self, StorageError> {
        Ok(serde_json::from_str(json_str)?)
    }
    
    pub fn get_progress_json(&self) -> Result<String, StorageError> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_ok() {
        let json = r#"{
            "m4b_data": {
                "some_hash": {
                    "path": "test/file",
                    "cur_timestamp": 120000,
                    "last_updated": 1680000000
                }
            }
        }"#;
    
        let storage = Storage::load(json);
    
        assert!(storage.is_ok());
    
        let storage = storage.unwrap();
    
        assert_eq!(storage.m4b_data.len(), 1);
    
        let data = storage.m4b_data.get("some_hash").unwrap();
    
        assert_eq!(data.path, "test/file");
        assert_eq!(data.cur_timestamp, 120000);
        assert_eq!(data.last_updated, 1680000000);
    }
    
    #[test]
    fn load_invalid_json_returns_err() {
        let json = r#"{
            "m4b_data": {
                "some_hash": {
                    "path": "test/file",
                    "cur_timestamp": "wrong_type",
                    "last_updated": 1680000000
                }
            }
        }"#;
        let storage = Storage::load(json);
    
        assert!(storage.is_err());
    }
    
    #[test]
    fn get_progress_json_returns_ok() {
        let json = r#"{"m4b_data":{"abc123":{"path":"test/file","cur_timestamp":120000,"last_updated":1680000000}}}"#;
        let storage = Storage::load(json).unwrap();
        let serialized = storage.get_progress_json().unwrap();
        let deserialized = Storage::load(&serialized).unwrap();
    
        assert_eq!(storage, deserialized);
    }
}
