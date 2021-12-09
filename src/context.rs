use serde::{Deserialize, Serialize};
use wasmparser::{Parser, Payload};

#[derive(Debug, Serialize, Deserialize)]
pub struct SycContext {
  pub literal_offsets: Vec<(usize, usize)>,
}

impl SycContext {
  pub fn new() -> Self {
    Self {
      literal_offsets: Vec::new(),
    }
  }
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
