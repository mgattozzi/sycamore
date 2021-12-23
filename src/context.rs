use serde::{Deserialize, Serialize};
use wasmparser::{Parser, Payload};
use wasmtime_wasi::WasiCtx;

/// `SycContext` is what hosts all of the needed context to run a sycamore
/// program. As of right now this is just `WasiCtx` which gets added in at
/// runtime, but more might be added in the future. It gets encoded into the
/// custom section of the wasm binary and can be retrieved from it to run the
/// program.
#[derive(Serialize, Deserialize)]
pub struct SycContext {
  #[serde(skip)]
  pub wasi: Option<WasiCtx>,
}

impl SycContext {
  /// Create a new `SycContext`
  pub fn new() -> Self {
    Self { wasi: None }
  }
  /// Retrieve a `SycContext` from a sycamore binary.
  pub fn from_sycamore_binary(wasm: &[u8]) -> Self {
    bincode::deserialize(
      Parser::new(0)
        .parse_all(&wasm)
        .find_map(|payload| match payload.unwrap() {
          Payload::CustomSection { data, .. } => Some(data),
          _ => None,
        })
        .unwrap(),
    )
    .unwrap()
  }
}
