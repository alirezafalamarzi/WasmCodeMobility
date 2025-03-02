use std::{fs, error::Error};
use wasmtime::{component::{ResourceTable, bindgen, Component, Linker}, *};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
bindgen!("myworld" in "../guest/wit/witfile.wit");

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}
impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::new(Config::new().wasm_component_model(true))?;
    let bytes = fs::read("../guest/target/wasm32-wasip2/release/guest.wasm")?;
    let component = Component::new(&engine, &bytes)?;
    let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();


    let mut linker = Linker::<MyState>::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let mut store = wasmtime::Store::new(&engine,
         MyState {
            ctx: wasi_ctx,
            table: ResourceTable::new(),
         });
    let instance = linker.instantiate(&mut store, &component)?;

    let user1 = UserData {
        first_name: String::from("Ali"),
        last_name: String::from("Toby"),
        age: 24,
        grades: vec![12, 13, 15],
    };

    let functions = Myworld::instantiate(&mut store, &component, &linker)?;
    let result2 = functions.call_change_user(&mut store, &user1).unwrap();
    let result3 = functions.call_get_name(&mut store, &user1).unwrap();

    println!("Greeting: {:?}", result2.grades[1] + result2.grades[2]);
    println!("Greeting: {:?}", result3);
    Ok(())
}