use crate::{
  standard_library::{self, Print, PrintLn},
  Statement,
};
use std::collections::HashMap;
use wasm_encoder::*;

pub struct Codegen {
  pub debug: bool,
  pub stmt: Vec<Statement>,
  pub main_mod: Module,
  pub imports: ImportSection,
  pub data: DataSection,
  pub memory: MemorySection,
  pub name: Name,
  pub types: TypeSection,
  pub functions: FunctionSection,
  pub exports: ExportSection,
  pub codes: CodeSection,
  pub literal_map: HashMap<String, i32>,
  pub fn_map: HashMap<String, u32>,
  pub current_func: Function,
}

pub struct Name {
  pub function_names: NameMap,
  pub memory_names: NameMap,
  pub type_names: NameMap,
  pub local_names: IndirectNameMap,
}

impl Name {
  pub fn new() -> Self {
    Self {
      function_names: NameMap::new(),
      memory_names: NameMap::new(),
      type_names: NameMap::new(),
      local_names: IndirectNameMap::new(),
    }
  }

  pub fn make_section(&self) -> NameSection {
    let mut section = NameSection::new();
    // Set all of the debug names
    section.module("main");
    section.functions(&self.function_names);
    section.locals(&self.local_names);
    section.types(&self.type_names);
    section.memories(&self.memory_names);
    section
  }
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
      name: Name::new(),
      types: TypeSection::new(),
      functions: FunctionSection::new(),
      exports: ExportSection::new(),
      codes: CodeSection::new(),
      literal_map: HashMap::new(),
      fn_map: HashMap::new(),
      current_func: Function::new(Vec::new()),
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
    self.name.memory_names.append(0, "main_memory");

    // Imports *must* happen first for the func number to be correct. We start
    // off with stdlib so that we import everything we need. Then when we
    // actually can import multiple files we would import other modules here and
    // link them.
    standard_library::import_stdlib(&mut self);

    // Setup the function map and types after our import so that we can make
    // calls to them properly everywhere. Also create all of our string literals
    // before hand.
    for stmt in self.stmt.iter() {
      if let Statement::StateDefn {
        name, statements, ..
      } = stmt
      {
        // All states are the void type for now until we deal with args
        self.types.function(Vec::new(), Vec::new());
        let name = name.as_str().to_string();
        let idx = self.fn_map.len() as u32;
        self.name.function_names.append(idx, &name);
        self.name.type_names.append(idx, &name);
        self.fn_map.insert(name, idx);

        // Setup literals ahead of time in our data section
        for stmt in statements {
          match stmt {
            Statement::Print(Print(literal)) | Statement::PrintLn(PrintLn(literal)) => {
              let offset = {
                let mut offset = 0;
                for lit in self.literal_map.keys() {
                  offset += lit.len();
                }
                if offset > 0 {
                  offset += 1;
                }
                offset as i32
              };
              self
                .data
                .active(0, &Instruction::I32Const(offset), literal.as_str().bytes());
              self.literal_map.insert(literal.as_str().into(), offset);
            }
            _ => {}
          }
        }
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
    self.main_mod.section(&self.name.make_section());

    // Create and validate
    let debug = self.debug;
    let wasm = self.finish();
    if debug {
      println!("---------------- Codegen WAT Output ----------------");
      let wat = wabt::Wasm2Wat::new()
        .read_debug_names(true)
        .convert(&wasm)
        .map(|buf| String::from_utf8(buf.as_ref().to_vec()).unwrap())
        .unwrap();
      println!("{}", wat);
    }

    // We should *not* generate incorrect wasm at all. This checks that we don't
    // do that.
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
