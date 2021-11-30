use crate::{Codegen, Generate, StdLib, StrLit};
use wasm_encoder::*;
use wasmtime::{AsContext, Caller, Func, Store};

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
      offset as i32
    };
    codegen
      .data
      .active(0, &Instruction::I32Const(offset), literal.as_str().bytes());
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(offset));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(offset + literal.len() as i32));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::Call(1));
    codegen.literal_table.push(literal.as_str().into());
  }
}

impl StdLib for PrintLn {
  fn import(codegen: &mut Codegen) {
    let fn_num = codegen.fn_map.len() as u32;
    codegen
      .types
      .function(vec![ValType::I32, ValType::I32], Vec::new());
    codegen.fn_map.entry("println".into()).or_insert(fn_num);
    codegen
      .imports
      .import("std", Some("println"), EntityType::Function(fn_num));
    codegen.name.function_names.append(fn_num, "println");
    codegen.name.type_names.append(fn_num, "println");
    codegen.name.local_names.append(fn_num, &NameMap::new());
  }
  fn func(store: &mut Store<()>) -> Func {
    Func::wrap(
      store,
      |mut caller: Caller<'_, ()>, offset: i32, len: i32| {
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
    )
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
      offset as i32
    };
    codegen
      .data
      .active(0, &Instruction::I32Const(offset), literal.as_str().bytes());
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(offset));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::I32Const(offset + literal.len() as i32));
    codegen
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&Instruction::Call(0));
    codegen.literal_table.push(literal.as_str().into());
  }
}

impl StdLib for Print {
  fn import(codegen: &mut Codegen) {
    let fn_num = codegen.fn_map.len() as u32;
    codegen
      .types
      .function(vec![ValType::I32, ValType::I32], Vec::new());
    codegen.fn_map.entry("print".into()).or_insert(fn_num);
    codegen
      .imports
      .import("std", Some("print"), EntityType::Function(fn_num));
    codegen.name.function_names.append(fn_num, "print");
    codegen.name.type_names.append(fn_num, "print");
    codegen.name.local_names.append(fn_num, &NameMap::new());
  }
  fn func(store: &mut Store<()>) -> Func {
    Func::wrap(
      store,
      |mut caller: Caller<'_, ()>, offset: i32, len: i32| {
        let data = &caller
          .get_export("main_memory")
          .unwrap()
          .into_memory()
          .unwrap()
          .data(caller.as_context());
        print!(
          "{}",
          std::str::from_utf8(&data[offset as usize..len as usize]).unwrap()
        );
      },
    )
  }
}

pub fn import_stdlib(codegen: &mut Codegen) {
  Print::import(codegen);
  PrintLn::import(codegen);
}
