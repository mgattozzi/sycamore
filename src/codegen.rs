use crate::{Print, PrintLn, Statement};
use std::collections::HashMap;
use wasm_encoder::*;

pub struct Codegen {
  pub debug: bool,
  pub stmt: Vec<Statement>,
  pub main_mod: Module,
  pub imports: ImportSection,
  pub data: DataSection,
  pub memory: MemorySection,
  pub types: TypeSection,
  pub functions: FunctionSection,
  pub exports: ExportSection,
  pub codes: CodeSection,
  pub literal_table: Vec<String>,
  pub fn_map: HashMap<String, usize>,
  pub current_func: Option<Function>,
}

impl Codegen {
  pub fn new(stmt: Vec<Statement>, debug: bool) -> Self {
    Self {
      debug,
      stmt,
      main_mod: Module::new(),
      imports: ImportSection::new(),
      data: DataSection::new(),
      memory: MemorySection::new(),
      types: TypeSection::new(),
      functions: FunctionSection::new(),
      exports: ExportSection::new(),
      codes: CodeSection::new(),
      literal_table: Vec::new(),
      fn_map: HashMap::new(),
      current_func: None,
    }
  }

  fn finish(self) -> Vec<u8> {
    self.main_mod.finish().to_vec()
  }

  pub fn generate(mut self) -> Vec<u8> {
    Print::import(&mut self);
    PrintLn::import(&mut self);

    self.memory.memory(MemoryType {
      minimum: 1,
      maximum: None,
      memory64: false,
    });
    // print function type
    self
      .types
      .function(vec![ValType::I32, ValType::I32], Vec::new());
    // void function type
    self.types.function(Vec::new(), Vec::new());

    self.exports.export("main_memory", Export::Memory(0));

    for fn_name in self.stmt.iter().filter_map(|a| {
      if let Statement::StateDefn { name, .. } = a {
        Some(name)
      } else {
        None
      }
    }) {
      self
        .fn_map
        .insert(fn_name.as_str().into(), self.fn_map.len());
    }

    for statement in self.stmt.clone().iter() {
      statement.generate(&mut self);
    }

    // Set the sections in the right order
    self.main_mod.section(&self.types);
    self.main_mod.section(&self.imports);
    self.main_mod.section(&self.functions);
    self.main_mod.section(&self.memory);
    self.main_mod.section(&self.exports);
    self.main_mod.section(&self.codes);
    self.main_mod.section(&self.data);

    // Create and validate
    let debug = self.debug;
    let wasm = self.finish();
    if debug {
      println!("---------------- Codegen WAT Output ----------------");
      println!("{}", wabt::wasm2wat(&wasm).unwrap_or(String::new()));
    }

    if let Err(e) = wasmparser::validate(&wasm) {
      panic!("{}", e);
    }

    wasm
  }
}

pub trait Generate {
  fn generate(&self, codegen: &mut Codegen);
}

pub trait StdLib: Generate {
  fn import(codegen: &mut Codegen);
  fn func(store: &mut wasmtime::Store<()>) -> wasmtime::Func;
}
