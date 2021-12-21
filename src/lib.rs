mod codegen;
mod context;
mod parser;
mod types;
mod wasi;

use crate::{codegen::Codegen, context::SycContext, parser::SycParser, wasi::wasi_linker};
use std::{error::Error, fs, path::PathBuf};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

pub fn build(path: &mut PathBuf, debug: bool) -> Result<Vec<u8>, Box<dyn Error>> {
  let input = fs::read_to_string(&path)?;
  if debug {
    println!("------------------ Sycamore Input ------------------");
    println!("{}", input);
  }
  let parsed = SycParser::new(&input).parse();
  let wasm = Codegen::new(parsed, debug).generate();

  path.set_extension("wasm");
  fs::write(path.file_name().unwrap(), &wasm)?;
  Ok(wasm)
}

pub fn run(wasm: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }

  let engine = Engine::default();
  let mut linker = Linker::new(&engine);
  wasi_linker(&mut linker)?;
  let mut ctx = SycContext::from_sycamore_binary(&wasm);
  ctx.wasi = Some(
    WasiCtxBuilder::new()
      .inherit_stdio()
      .inherit_args()?
      .build(),
  );

  let module = Module::new(&engine, wasm)?;
  let mut store = Store::new(&engine, ctx);
  let instance = linker.instantiate(&mut store, &module)?;
  let main = instance.get_typed_func::<(), (), _>(&mut store, "_start")?;
  main.call(&mut store, ())?;

  Ok(())
}
