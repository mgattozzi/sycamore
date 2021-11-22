mod codegen;
mod parser;
mod standard_library;

pub use crate::standard_library::*;
pub use codegen::*;
pub use parser::*;

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

  path.set_extension("wasm");
  fs::write(path.file_name().unwrap(), &wasm)?;
  Ok(wasm)
}

pub fn run(wasm: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }
  let engine = Engine::default();
  let module = Module::new(&engine, wasm)?;
  let mut store = Store::new(&engine, ());
  let std_funcs = [
    print(&mut store).into(),
    // add println
    println(&mut store).into(),
  ];
  let instance = Instance::new(&mut store, &module, &std_funcs)?;
  let main = instance.get_typed_func::<(), (), _>(&mut store, "main")?;
  main.call(&mut store, ())?;

  Ok(())
}
