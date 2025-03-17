use std::{fs, error::Error};
use wasmtime::{component::{ResourceTable, bindgen, Component, Linker}, *};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use reqwest::blocking::get;
bindgen!("myworld" in "../guest-cache/wit/witfile.wit");

struct HostComponent;

// Implementation of the host interface defined in the wit file.
impl host::Host for HostComponent {
    fn multiply(&mut self, a: f32, b: f32) -> f32 {
        a * b
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
    // let bytes = fs::read("../guest/target/wasm32-wasip2/release/guest.wasm")?;
    let bytes = fs::read("../guest-cache/target/wasm32-wasip2/release/guest_cache.wasm")?;
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
    let result1 = functions.call_get_or_fetch(&mut store, "./data.json", "http://localhost:8888", 1000);
    let result2 = functions.call_get_or_fetch(&mut store, "./data.json", "http://localhost:8888/leisure_data.csv", 1000);
    let result3 = functions.call_get_or_fetch(&mut store, "./data.json", "http://localhost:8888/config.json", 1000);
    let result4 = functions.call_get_or_fetch(&mut store, "./data.json", "http://localhost:8888/chart.plugin.js", 1000);
    let result5 = functions.call_get_or_fetch(&mut store, "./data.json", "http://localhost:8888/q1.jpg", 1000);


    println!("{:?}", result1.unwrap().unwrap());
    println!("{:?}", result2.unwrap().unwrap());
    println!("{:?}", result3.unwrap().unwrap());
    println!("{:?}", result4.unwrap().unwrap());
    // println!("{:?}", result5.unwrap().unwrap());

    // let mut host = HostComponent{};
    // host.write_to_file(result5.unwrap().unwrap(), String::from("q1.jpg"));
    Ok(())
}

