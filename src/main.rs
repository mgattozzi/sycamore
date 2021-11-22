use clap::Parser;
use libsyc::{build, run};
use std::{error::Error, path::PathBuf};

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
