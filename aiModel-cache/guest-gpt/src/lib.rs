use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::io::prelude::*;

const MAX_ITEMS: usize = 1000;

#[derive(Serialize, Deserialize)]
struct Entry {
    model: String,
    prompt: String,
    response: String,
    context: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
struct Data {
    entries: VecDeque<Entry>,
}

impl Data {
    fn new() -> Self {
        Data {
            entries: VecDeque::new(),
        }
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
    fn load_cache(&self) -> Data {
        let contents = host::read_from_file(&self.file_path);
        if let Ok(data) = serde_json::from_str(&contents) {
            return data;
        }
        Data::new()
    }

    /// Saves the cache data to disk.
    fn save_cache(&self, data: &Data) {
        if let Ok(json) = serde_json::to_string(data) {
            host::write_to_file(&json, &self.file_path);
        }
    }

    /// Adds or updates a cache entry.
    /// For expiry and last_modified, pass None if you do not wish to set them.
    pub fn add_response(&self, model: &str, prompt: &str, response: &str, context: &Vec<u64>) {
        let mut data = self.load_cache();
        let entry = Entry {
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            context: context.clone(),
        };
        if data.entries.len() >= MAX_ITEMS {
            data.entries.pop_front(); // Remove the oldest item
        }
        data.entries.push_back(entry);
        self.save_cache(&data);
    }

    /// Retrieves a cached response if it exists and is fresh.
    /// Returns None on a cache miss or if the entry is stale.
    pub fn get_response(&self, model: &str, prompt: &str) -> Option<String> {
        let data = self.load_cache();
        let prompt_lower = prompt.to_lowercase();
        let model_lower = model.to_lowercase();
        data.entries
            .iter()
            .find(|entry| {
                entry.model.to_lowercase() == model_lower
                    && entry.prompt.to_lowercase().contains(&prompt_lower)
            })
            .map(|entry| entry.response.clone())
    }

    pub fn get_latest_context(&self, model: &str) -> Option<Vec<u64>> {
        let data = self.load_cache();
        data.entries
            .iter()
            .rev()
            .find(|entry| entry.model.eq_ignore_ascii_case(model))
            .map(|entry| entry.context.clone())
    }

    // /// Invalidates a specific cache entry.
    // pub fn invalidate(&self, entry: &Entry) {
    //     let mut data = self.load_cache();
    //     data.entries.remove(entry);
    //     self.save_cache(&data);
    // }

    /// Clears all cache entries.
    pub fn clear(&self) {
        let data = Data::new();
        self.save_cache(&data);
    }

    fn parse_ollama_stream(&self, json_stream: &str) -> (String, Vec<u64>) {
        let mut response = String::new();
        let mut context = Vec::new();

        for line in json_stream.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(json) => {
                    if let Some(text) = json["response"].as_str() {
                        response.push_str(text);
                    }

                    if json.get("done") == Some(&Value::Bool(true)) {
                        if let Some(ctx) = json.get("context").and_then(|v| v.as_array()) {
                            context = ctx.iter().filter_map(|v| v.as_u64()).collect();
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to parse JSON line: {}", err);
                    continue;
                }
            }
        }

        (response, context)
    }
}

wit_bindgen::generate!({
    path: "wit",
    world: "myworld",
});

struct MyHost;

impl Guest for MyHost {
    fn get_or_fetch(file_path: String, model: String, prompt: String) -> Option<String> {
        let cache = FileCache::new(file_path);
        if let Some(cached) = cache.get_response(&model, &prompt) {
            println!("Cache hit for model: {}, prompt {}", model, prompt);
            Some(cached)
        } else {
            println!(
                "Cache miss or stale entry for model: {}, prompt: {}. Fetching from network...",
                model, prompt
            );

            let latest_context = cache.get_latest_context(&model).unwrap_or_default();
            println!("{:?}", latest_context);
            match host::manual_get(&model, &prompt, &latest_context) {
                Some(response_raw) => {
                    let (response, new_context) = cache.parse_ollama_stream(&response_raw);
                    // For demonstration, we set no expiry.
                    // In practice, you might set an expiry based on headers.
                    cache.add_response(&model, &prompt, &response, &new_context);
                    println!("Response caught! returning...");
                    Some(response)
                }
                None => {
                    eprintln!(
                        "Failed to fetch response from network for model: {}, prompt: {}",
                        model, prompt
                    );
                    None
                }
            }
        }
    }
}

export!(MyHost);

