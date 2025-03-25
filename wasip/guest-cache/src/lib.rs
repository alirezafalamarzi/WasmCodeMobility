#![feature(wasip1)]

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use std::net::TcpStream;
use std::io::prelude::*;
use url::Url;


use reqwest::blocking::get;

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

    fn manual_get(&mut self, url: String) -> Option<String> {
        // Send the GET request and return None if there's an error
        let response = get(&url).ok()?
;

        // Check that the request was successful
        if !response.status().is_success() {
            return None;
        }

        // Read the response body as bytes, returning None on error
        let text = response.text().ok()?;

        println!("Fetched {} bytes", text.len());
        Some(text)
    }

    fn write_to_file(&mut self, data: String, file_name: String) {
        if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_name)
    {
        let _ = file.write_all(data.as_bytes());
    }
    }

    fn read_from_file(&mut self, file_name: String) -> String {
        match File::open(&file_name) {
            Ok(mut file) => {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    return contents;
                } else {
                    String::from("")
                }
            },
            Err(_) => String::from(""),
        }
    }

    /// Loads the cache data from disk.
    /// If the file can’t be read or parsed, returns an empty cache.
    fn load_cache(&self) -> CacheData {
        let contents = self.read_from_file(&self.file_path);
        if let Ok(data) = serde_json::from_str(&contents) {
            return data;
        }
        CacheData::new()
    }

    /// Saves the cache data to disk.
    fn save_cache(&self, data: &CacheData) {
        if let Ok(json) = serde_json::to_string(data) {
            self.write_to_file(&json, &self.file_path);
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

    pub fn manual_get(&self, url: &str) -> Option<String> {
        // Parse the URL
        let url = Url::parse(url).ok()?;
        let host = url.host_str()?;
        let port = url.port_or_known_default()?;
        let addr = format!("{}:{}", host, port);

        // Determine the path (and query, if any)
        let mut path = url.path().to_string();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        if path.is_empty() {
            path = "/".to_string();
        }

        // Connect to the host
        let mut stream = TcpStream::connect(&addr).ok()?;

        // Construct a minimal HTTP GET request.
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        stream.write_all(request.as_bytes()).ok()?;

        // Read the entire response.
        let mut response = Vec::new();
        stream.read_to_end(&mut response).ok()?;
        let response = String::from_utf8(response).ok()?;

        // Separate HTTP headers from the body.
        if let Some(pos) = response.find("\r\n\r\n") {
            let body = &response[(pos + 4)..];
            return Some(body.to_string());
        }
        Some(response)
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
            match self.manual_get(&key) {
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





// fn main() {
//     // Specify the file where the cache will be stored.
//     let file_path = "cache_data.json";

//     let getter = Getter::new();
//     // Use a URL as the key.
//     let key = "https://www.example.com";
//     // For demonstration, we use 0 as the current time so that the expiry check passes.
//     let current_time = 0;

//     match getter.get_or_fetch(file_path, key, current_time) {
//         Some(response) => {
//             println!("Response:\n{}", response);
//         },
//         None => {
//             println!("Failed to fetch the response.");
//         }
//     }
// }





