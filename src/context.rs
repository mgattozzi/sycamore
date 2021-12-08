use wasmparser::{Parser, Payload};
pub struct SycContext {
  // lol
}

impl SycContext {
  pub fn from_sycamore_binary(wasm: &[u8]) -> Self {
    let data = Parser::new(0)
      .parse_all(&wasm)
      .find_map(|payload| match payload.unwrap() {
        Payload::CustomSection { data, .. } => Some(data),
        _ => None,
      })
      .unwrap();
    Self {}
  }
}
