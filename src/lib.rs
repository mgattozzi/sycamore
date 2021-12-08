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
  let cwasm = Codegen::new(parsed, debug).generate();
  path.set_extension("csm");
  fs::write(path.file_name().unwrap(), &cwasm)?;
  Ok(cwasm)
}

/// Run a sycamore program given a valid input of bytes
pub fn run(csm: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }
  let engine = Engine::default();
  let module = Module::new(&engine, &csm)?;
  let mut linker = Linker::new(&engine);
  wasi_linker(&mut linker)?;
  let mut ctx = SycContext::from_sycamore_binary(&csm);
  ctx.wasi = Some(
    WasiCtxBuilder::new()
      .inherit_stdio()
      .inherit_args()?
      .build(),
  );

  let mut store = Store::new(&engine, ctx);
  let instance = linker.instantiate(&mut store, &module)?;
  let main = instance.get_typed_func::<(), (), _>(&mut store, "_start")?;
  main.call(&mut store, ())?;

  Ok(())
}
