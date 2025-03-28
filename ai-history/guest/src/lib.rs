use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

const MAX_ITEMS: usize = 1000;

// An cached entry from the AI model
#[derive(Serialize, Deserialize)]
struct Entry {
    model: String,
    prompt: String,
    response: String,
    context: Vec<u64>,
}

// The list of entries as a doubly linked list
#[derive(Serialize, Deserialize)]
struct Data {
    entries: VecDeque<Entry>,
}

// Just for creating new data
impl Data {
    fn new() -> Self {
        Data {
            entries: VecDeque::new(),
        }
    }
}

// To structure our functions that nead file and serialization operations.
pub struct FileCache {
    file_path: String,
}

impl FileCache {
    // Create a new file cache object that only includes a path as string
    pub fn new(file_path: impl Into<String>) -> Self {
        FileCache {
            file_path: file_path.into(),
        }
    }

    // Loads the cache data from disk.
    fn load_cache(&self) -> Data {
        let contents = host::read_from_file(&self.file_path);
        // If the file can’t be read or parsed, returns an empty cache.
        if let Ok(data) = serde_json::from_str(&contents) {
            return data;
        }
        Data::new()
    }

    // Saves the cache data to disk.
    fn save_cache(&self, data: &Data) {
        if let Ok(json) = serde_json::to_string(data) {
            host::write_to_file(&json, &self.file_path);
        }
    }

    // Adds or updates a cache entry.
    pub fn add_response(&self, model: &str, prompt: &str, response: &str, context: &Vec<u64>) {
        let mut data = self.load_cache();
        let entry = Entry {
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            context: context.clone(),
        };
        // If the cache list was overflowing the maximum allowed cache entries
        // then remove the oldest entry
        if data.entries.len() >= MAX_ITEMS {
            data.entries.pop_front(); // Remove the oldest item
        }
        // Add the entry to the end of the list
        data.entries.push_back(entry);
        self.save_cache(&data);
    }

    // Retrieves a cached response if it exists and is fresh.
    // Returns None on a cache miss or if the entry is stale.
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

    // Retrieves the latest (newest) context from the cache
    pub fn get_latest_context(&self, model: &str) -> Option<Vec<u64>> {
        let data = self.load_cache();
        data.entries
            .iter()
            .rev()
            .find(|entry| entry.model.eq_ignore_ascii_case(model))
            .map(|entry| entry.context.clone())
    }

    // Clears all cache entries.
    pub fn clear(&self) {
        let data = Data::new();
        self.save_cache(&data);
    }

    // Extracts the response and the context from a json ollama response got from the host
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
                    continue;
                }
            }
        }
        // Return the extracted response and context
        (response, context)
    }
}

// Generate rust code from WIT
wit_bindgen::generate!({
    path: "wit",
    world: "chat",
});

// The exported host struct (For WIT)
struct MyHost;

// Implement the Guest trait for the exported struct
impl Guest for MyHost {
    // Get the response to a prompt from cache if available otherwise get it from the AI model
    fn ask(file_path: String, model: String, prompt: String) -> Option<String> {
        let cache = FileCache::new(file_path);
        // If the prompt was previously asked and existed in the cache
        if let Some(cached) = cache.get_response(&model, &prompt) {
            Some(cached)
        // If the prompt didn't exist in the cache
        } else {
            let latest_context = cache.get_latest_context(&model).unwrap_or_default();
            // Get the response from the AI model
            match host::ask_model(&model, &prompt, &latest_context) {

                Some(response_raw) => {
                    let (response, new_context) = cache.parse_ollama_stream(&response_raw);
                    cache.add_response(&model, &prompt, &response, &new_context);
                    Some(response)
                }
                None => {
                    None
                }
            }
        }
    }
}

export!(MyHost);

