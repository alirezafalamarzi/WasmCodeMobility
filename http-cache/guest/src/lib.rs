use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    body: String,
    expiry: Option<u64>,
    etag: Option<String>,
    last_modified: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct CacheData {
    entries: HashMap<String, CacheEntry>,
}

impl CacheData {
    fn new() -> Self {
        CacheData { entries: HashMap::new() }
    }
}

pub struct FileCache {
    file_path: String,
}


impl FileCache {
    /// Creates a new file‑backed cache given the file path.
    pub fn new(file_path: impl Into<String>) -> Self {
        FileCache {
            file_path: file_path.into(),
        }
    }

    /// Loads the cache data from disk.
    /// If the file can’t be read or parsed, returns an empty cache.
    fn load_cache(&self) -> CacheData {
        let contents = host::read_from_file(&self.file_path);
        if let Ok(data) = serde_json::from_str(&contents) {
            return data;
        }
        CacheData::new()
    }

    /// Saves the cache data to disk.
    fn save_cache(&self, data: &CacheData) {
        if let Ok(json) = serde_json::to_string(data) {
            host::write_to_file(&json, &self.file_path);
        }
    }

    /// Adds or updates a cache entry.
    /// For expiry and last_modified, pass None if you do not wish to set them.
    pub fn add_response(
        &self,
        key: &str,
        body: &str,
        expiry: Option<u64>,
        etag: Option<&str>,
        last_modified: Option<u64>,
    ) {
        let mut data = self.load_cache();
        let entry = CacheEntry {
            body: body.to_string(),
            expiry,
            etag: etag.map(String::from),
            last_modified,
        };
        data.entries.insert(key.to_string(), entry);
        self.save_cache(&data);
    }

    /// Retrieves a cached response if it exists and is fresh.
    /// Returns None on a cache miss or if the entry is stale.
    pub fn get_response(&self, key: &str, current_time: u64) -> Option<String> {
        let data = self.load_cache();
        if let Some(entry) = data.entries.get(key) {
            if let Some(expiry) = entry.expiry {
                if current_time > expiry {
                    return None;
                }
            }
            Some(entry.body.clone())
        } else {
            None
        }
    }

    /// Invalidates a specific cache entry.
    pub fn invalidate(&self, key: &str) {
        let mut data = self.load_cache();
        data.entries.remove(key);
        self.save_cache(&data);
    }

    /// Clears all cache entries.
    pub fn clear(&self) {
        let data = CacheData::new();
        self.save_cache(&data);
    }

}



wit_bindgen::generate!({
    path: "wit",
    world: "myworld",
});


struct MyHost;

impl Guest for MyHost {
    fn get_or_fetch(file_path: String, key: String, current_time: u64) -> Option<String> {
        let cache = FileCache::new(file_path);
        if let Some(cached) = cache.get_response(&key, current_time) {
            println!("Cache hit for {}", key);
            Some(cached)
        } else {
            println!("Cache miss or stale entry for {}. Fetching from network...", key);
            match host::manual_get(&key) {
                Some(body) => {
                    // For demonstration, we set no expiry.
                    // In practice, you might set an expiry based on headers.
                    cache.add_response(&key, &body, None, None, None);
                    Some(body)
                },
                None => {
                    eprintln!("Failed to fetch response from network for {}", key);
                    None
                }
            }
        }
    }
}

export!(MyHost);
