use clap::Parser;
use libsyc::{Codegen, SycParser};
use std::{error::Error, fs, path::PathBuf};
use wasmtime::*;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Parser)]
#[clap(
  name = env!("CARGO_BIN_NAME"),
  version = env!("CARGO_PKG_VERSION"),
  author = env!("CARGO_PKG_AUTHORS"),
  about = env!("CARGO_PKG_DESCRIPTION"))
]
struct Opts {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
  /// Compile sycamore code to wasm
  Build { path: PathBuf },
  /// Compile sycamore code to wasm and run it
  Run {
    path: PathBuf,
    #[clap(short, long)]
    debug: bool,
  },
}

fn main() -> Result<(), Box<dyn Error>> {
  let opts = Opts::parse();

  match opts.subcmd {
    SubCommand::Build { mut path } => build(&mut path, false).map(drop),
    SubCommand::Run { mut path, debug } => {
      let wasm = build(&mut path, debug)?;
      run(wasm, debug)
    }
  }
}

fn build(path: &mut PathBuf, debug: bool) -> Result<Vec<u8>, Box<dyn Error>> {
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

fn run(wasm: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
  if debug {
    println!("------------------ Code Execution ------------------");
  }
  let engine = Engine::default();
  let module = Module::new(&engine, wasm)?;
  let mut store = Store::new(&engine, 0);
  let print = Func::wrap(
    &mut store,
    move |mut caller: Caller<'_, u32>, offset: i32, len: i32| {
      let data = &caller
        .get_export("main_memory")
        .unwrap()
        .into_memory()
        .unwrap()
        .data(caller.as_context());
      println!(
        "{}",
        std::str::from_utf8(&data[offset as usize..len as usize]).unwrap()
      );
    },
  );
  let instance = Instance::new(&mut store, &module, &[print.into()])?;
  let main = instance.get_typed_func::<(), (), _>(&mut store, "main")?;
  main.call(&mut store, ())?;

  Ok(())
}
