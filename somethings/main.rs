use std::{fs, path::Path, error::Error};
use wasmtime::{component::{ResourceTable, bindgen, Component, Linker}, *};
use anyhow::Context;
use wasmtime_wasi::bindings::{cli::stderr::add_to_linker, exports::wasi, sync::Command};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
bindgen!("myworld" in "../guest/wit/witfile.wit");

struct HostComponent;
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

// fn convert_to_component(path: impl AsRef<Path>) -> Result<Vec<u8>> {
//     let bytes = &fs::read(&path).context("failed to read input wasm file")?;
//     wit_component::ComponentEncoder::default()
//     .module(&bytes)?
//     .encode()
// }
fn main() -> Result<(), Box<dyn Error>> {
    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::new(Config::new().wasm_component_model(true))?;

    // let component = convert_to_component("../guest/target/wasm32-wasip2/release/guest.wasm")?;
    // let component = Component::from_binary(&engine, &component)?;
    let bytes = fs::read("../guest/target/wasm32-wasip2/release/guest.wasm")?;
    let component = Component::new(&engine, &bytes)?;

    // A `Store` is what will own instances, functions, globals, etc. All wasm
    // items are stored within a `Store`, and it's what we'll always be using to
    // interact with the wasm world. Custom data can be stored in stores but for
    // now we just use `()`.
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
    let func = instance.get_func(&mut store, "to-string").expect("to_string function not found!");

    // let answer = instance.get_func(&mut store, "answer")
        // .expect("`answer` was not an exported function");

    // There's a few ways we can call the `answer` `Func` value. The easiest
    // is to statically assert its signature with `typed` (in this case
    // asserting it takes no arguments and returns one i32) and then call it.
    // let answer = answer.typed::<(), i32>(&store)?;

    // And finally we can call our function! Note that the error propagation
    // with `?` is done to handle the case where the wasm function traps.
    // let result = answer.call(&mut store, ())?;
    // let mut result = [wasmtime::component::Val::String("".into())];
    // let user_val = wasmtime::component::Val::Record(vec![
    //     ("first-name".to_string(), wasmtime::component::Val::String(user1.first_name)),
    //     ("last-name".to_string(), wasmtime::component::Val::String(user1.last_name)),
    //     ("age".to_string(), wasmtime::component::Val::U32(user1.age as u32)),
    //     ("grades".to_string(), wasmtime::component::Val::List(
    //         user1.grades.into_iter()
    //             .map(|grade| wasmtime::component::Val::U32(grade))
    //             .collect()
    //     )),
    // ]);

    let convert = Myworld::instantiate(&mut store, &component, &linker)?;
    let result2 = convert.call_to_string(&mut store, &user1).unwrap();


    // let gr = String::from("Hello");
    // let mut result = [wasmtime::component::Val::Record(vec![
    //     ("first-name".to_string(), wasmtime::component::Val::String("".into())),
    //     ("last-name".to_string(), wasmtime::component::Val::String("".into())),
    //     ("age".to_string(), wasmtime::component::Val::U32(user1.age as u32)),
    //     ("grades".to_string(), wasmtime::component::Val::List(vec![])),
    // ])];
    // func.call(&mut store, &[user_val], &mut result)?;

    println!("Greeting: {:?}", result2);
    Ok(())
}