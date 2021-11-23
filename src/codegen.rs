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
      match statement {
        Statement::StateDefn {
          name,
          end,
          input,
          statements,
        } => {
          if name.as_str() == "main" {
            if !end {
              panic!("Main must be labelled an end state");
            }
            if !input.is_empty() {
              panic!("Main must have no arguments");
            }
            let void_function_index = 1;
            self.functions.function(void_function_index);
            self.exports.export("main", Export::Function(2));
          } else {
            let void_function_index = 1;
            self.functions.function(void_function_index);
          }

          let locals = Vec::new();
          self.current_func = Some(Function::new(locals));
          for stmt in statements {
            match stmt {
              Statement::Terminate => {
                // TODO: actually do something with this
              }
              Statement::Print(print) => print.generate(&mut self),
              Statement::PrintLn(println) => println.generate(&mut self),
              Statement::FnCall { name, .. } => {
                self.current_func.as_mut().map(|f| {
                  f.instruction(Instruction::Call(
                    *self.fn_map.get(name.as_str()).unwrap() as u32
                  ));
                  f
                });
              }
              Statement::StateDefn { .. } => panic!("Cannot define states inside a state"),
            }
          }
          self.current_func.as_mut().map(|f| {
            f.instruction(Instruction::End);
            f
          });

          let func = self.current_func.take().unwrap();
          self.codes.function(&func);
        }
        _ => panic!("Invalid only StateDefn are allowed"),
      }
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
