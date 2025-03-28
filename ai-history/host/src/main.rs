use std::{fs, error::Error};
use wasmtime::{component::{ResourceTable, bindgen, Component, Linker}, *};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};


use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use reqwest::blocking::Client;
use serde_json::json;
use std::io::{self, BufRead, BufReader};
use serde::Deserialize;

#[derive(Deserialize)]
struct OllamaStreamChunk {
    response: String,
    done: Option<bool>,
}

bindgen!("myworld" in "../guest-gpt/wit/witfile.wit");

struct HostComponent;

// Implementation of the host interface defined in the wit file.
impl host::Host for HostComponent {
    fn get_response(&mut self, model: String, prompt: String, context: Vec<u64>) -> Option<String> {
        let client = reqwest::blocking::Client::new();
        let api_url = "http://localhost:11434/api/generate";
        let mut payload = json!({
            "model": &model,
            "prompt": &prompt,
            "stream": false,
        });
        if !context.is_empty() {
            payload = json!({
                "model": &model,
                "prompt": &prompt,
                "stream": false,
                "context": &context,
            });
        }

        let response = client.post(api_url).json(&payload).send().ok()?;
        let reader = BufReader::new(response);
        let mut json_lines = String::new();
        for line in reader.lines().flatten() {
            if !line.trim().is_empty() {
                json_lines.push_str(&line);
                json_lines.push('\n');
            }
        }
        if json_lines.trim().is_empty() {
            // No response was returned by the model
            None
        } else {
            Some(json_lines)
        }

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
}

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
    host: HostComponent,
}
impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let bytes = fs::read("../guest-gpt/target/wasm32-wasip2/release/guest_cache.wasm")?;
    let component = Component::new(&engine, &bytes)?;
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();

    let mut store = wasmtime::Store::new(&engine,
         MyState {
            ctx: wasi_ctx,
            table: ResourceTable::new(),
            host: HostComponent {},
         });
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    host::add_to_linker(&mut linker, |state: &mut MyState| &mut state.host)?;

    let functions = Myworld::instantiate(&mut store, &component, &linker)?;


    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    loop {
        print!(">>> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        let bytes_read = handle.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            println!("Exiting...");
            break;
        }
        let result1 = functions.call_ask(&mut store, "./data.json", "mistral", &line);
        match &result1 {
            Ok(value) => println!("{:?}", value.as_ref().unwrap()),
            Err(err) => println!("{:?}", err),
        }
    }

    Ok(())
}
