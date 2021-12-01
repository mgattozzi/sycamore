mod codegen;
mod parser;
mod standard_library;
mod types;

pub use codegen::*;
pub use parser::*;
pub use types::*;

use crate::{codegen::StdLib, standard_library::*};
use std::{error::Error, fs, path::PathBuf};
use wasmtime::*;

pub fn build(path: &mut PathBuf, debug: bool) -> Result<Vec<u8>, Box<dyn Error>> {
  let input = fs::read_to_string(&path)?;
  if debug {
    println!("------------------ Sycamore Input ------------------");
    println!("{}", input);
  }
  let parsed = SycParser::new(&input).parse();
  let wasm = Codegen::new(parsed, debug).generate();
  let cwasm = Engine::new(&Config::new())?.precompile_module(&wasm)?;
  path.set_extension("csm");
  fs::write(path.file_name().unwrap(), &cwasm)?;
  Ok(cwasm)
}

pub fn run(csm: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }
  let engine = Engine::new(&Config::new())?;
  let module = unsafe { Module::deserialize(&engine, csm)? };
  run_inner(engine, module)
}

pub fn run_path(path: PathBuf, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }
  let engine = Engine::new(&Config::new())?;
  let module = unsafe { Module::deserialize_file(&engine, path)? };
  run_inner(engine, module)
}

fn run_inner(engine: Engine, module: Module) -> Result<(), Box<dyn Error>> {
  let mut store = Store::new(&engine, ());
  let std_funcs = [
    Print::func(&mut store).into(),
    // add println
    PrintLn::func(&mut store).into(),
  ];
  let instance = Instance::new(&mut store, &module, &std_funcs)?;
  let main = instance.get_typed_func::<(), (), _>(&mut store, "main")?;
  main.call(&mut store, ())?;

  Ok(())
}
