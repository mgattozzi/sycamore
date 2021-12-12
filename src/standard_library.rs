use crate::{context::SycContext, Codegen, Generate, StdLib, StrLit};
use wasm_encoder::*;
use wasmtime::{Caller, Func, Store};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintLn(StrLit);

impl PrintLn {
  pub fn new(lit: StrLit) -> Self {
    Self(lit)
  }
}

impl Generate for PrintLn {
  fn generate(&self, codegen: &mut Codegen) {
    let literal = &self.0;
    let offset = {
      let mut offset = 0;
      for lit in &codegen.literal_table {
        offset += lit.len();
      }
      if offset > 0 {
        offset += 1;
      }
      offset
    };
    codegen.data.active(
      0,
      &Instruction::I32Const(offset as i32),
      literal.as_str().bytes(),
    );
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(codegen.literal_table.len() as i32));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::Call(1));
    codegen
      .ctx
      .literal_offsets
      .push((offset, offset + literal.len()));
    codegen.literal_table.push(literal.as_str().into());
  }
}

impl StdLib for PrintLn {
  fn import(codegen: &mut Codegen) {
    let fn_num = codegen.fn_map.len() as u32;
    codegen.types.function(vec![ValType::I32], Vec::new());
    codegen.fn_map.entry("println".into()).or_insert(fn_num);
    codegen
      .imports
      .import("std", Some("println"), EntityType::Function(fn_num));
  }
  fn func(store: &mut Store<SycContext>) -> Func {
    Func::wrap(store, |mut caller: Caller<'_, SycContext>, lit: i32| {
      let (offset, len) = caller.data().literal_offsets[lit as usize];
      let data = &caller.get_memory("main_memory");
      println!("{}", std::str::from_utf8(&data[offset..len]).unwrap());
    })
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Print(StrLit);

impl Print {
  pub fn new(lit: StrLit) -> Self {
    Self(lit)
  }
}

impl Generate for Print {
  fn generate(&self, codegen: &mut Codegen) {
    let literal = &self.0;
    let offset = {
      let mut offset = 0;
      for lit in &codegen.literal_table {
        offset += lit.len();
      }
      if offset > 0 {
        offset += 1;
      }
      offset
    };
    codegen.data.active(
      0,
      &Instruction::I32Const(offset as i32),
      literal.as_str().bytes(),
    );
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(codegen.literal_table.len() as i32));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::Call(0));
    codegen
      .ctx
      .literal_offsets
      .push((offset, offset + literal.len()));
    codegen.literal_table.push(literal.as_str().into());
  }
}

impl StdLib for Print {
  fn import(codegen: &mut Codegen) {
    let fn_num = codegen.fn_map.len() as u32;
    codegen.types.function(vec![ValType::I32], Vec::new());
    codegen.fn_map.entry("print".into()).or_insert(fn_num);
    codegen
      .imports
      .import("std", Some("print"), EntityType::Function(fn_num));
  }
  fn func(store: &mut Store<SycContext>) -> Func {
    Func::wrap(store, |mut caller: Caller<'_, SycContext>, lit: i32| {
      let (offset, len) = caller.data().literal_offsets[lit as usize];
      let data = &caller.get_memory("main_memory");
      print!("{}", std::str::from_utf8(&data[offset..len]).unwrap());
    })
  }
}

pub fn import_stdlib(codegen: &mut Codegen) {
  Print::import(codegen);
  PrintLn::import(codegen);
}

pub trait CallerExt {
  fn get_memory(&mut self, mem: &str) -> &'_ [u8];
}

impl CallerExt for Caller<'_, SycContext> {
  fn get_memory(&mut self, mem: &str) -> &'_ [u8] {
    let memory = self.get_export(mem).unwrap().into_memory().unwrap();
    memory.data(self)
  }
}
