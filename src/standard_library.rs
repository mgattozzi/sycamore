use crate::{Codegen, Generate, StdLib, StrLit};
use wasm_encoder::*;
use wasmtime::{AsContext, Caller, Func, Store};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrintLn(pub StrLit);

impl PrintLn {
  pub fn new(lit: StrLit) -> Self {
    Self(lit)
  }
}

impl Generate for PrintLn {
  fn generate(&self, codegen: &mut Codegen) {
    let offset = *codegen.literal_map.get(self.0.as_str()).unwrap();
    codegen
      .current_func
      .instruction(&Instruction::I32Const(offset));
    codegen
      .current_func
      .instruction(&Instruction::I32Const(offset + self.0.len() as i32));
    codegen.current_func.instruction(&Instruction::Call(1));
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
pub struct Print(pub StrLit);

impl Print {
  pub fn new(lit: StrLit) -> Self {
    Self(lit)
  }
}

impl Generate for Print {
  fn generate(&self, codegen: &mut Codegen) {
    let offset = *codegen.literal_map.get(self.0.as_str()).unwrap();
    codegen
      .current_func
      .instruction(&Instruction::I32Const(offset));
    codegen
      .current_func
      .instruction(&Instruction::I32Const(offset + self.0.len() as i32));
    codegen.current_func.instruction(&Instruction::Call(0));
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
