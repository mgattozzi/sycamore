use crate::{context::SycContext, standard_library, Statement};
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
  pub fn_map: HashMap<String, u32>,
  pub current_func: Option<Function>,
  pub ctx: SycContext,
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
      ctx: SycContext::new(),
    }
  }

  fn finish(self) -> Vec<u8> {
    self.main_mod.finish().to_vec()
  }

  pub fn generate(mut self) -> Vec<u8> {
    self.memory.memory(MemoryType {
      minimum: 1,
      maximum: None,
      memory64: false,
    });

    self.exports.export("main_memory", Export::Memory(0));

    // Imports *must* happen first for the func number to be correct. We start
    // off with stdlib so that we import everything we need. Then when we
    // actually can import multiple files we would import other modules here and
    // link them.
    standard_library::import_stdlib(&mut self);

    // Setup the function map and types after our import so that we can make
    // calls to them properly everywhere.
    for stmt in self.stmt.iter() {
      if let Statement::StateDefn { name, .. } = stmt {
        // All states are the void type for now until we deal with args
        self.types.function(Vec::new(), Vec::new());
        self
          .fn_map
          .insert(name.as_str().into(), self.fn_map.len() as u32);
      }
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
    self.main_mod.section(&CustomSection {
      name: "SycContext",
      data: &bincode::serialize(&self.ctx).unwrap(),
    });
    // Create and validate
    let debug = self.debug;
    let wasm = self.finish();
    if debug {
      println!("---------------- Codegen WAT Output ----------------");
      println!("{}", wabt::wasm2wat(&wasm).unwrap_or(String::new()));
    }

    wasm
  }
}

pub trait Generate {
  fn generate(&self, codegen: &mut Codegen);
}

pub trait StdLib: Generate {
  fn import(codegen: &mut Codegen);
  fn func(store: &mut wasmtime::Store<SycContext>) -> wasmtime::Func;
}
