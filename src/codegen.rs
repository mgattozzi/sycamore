use crate::parser::*;
use std::collections::HashMap;
use wasm_encoder::*;

pub struct Codegen {
  debug: bool,
  stmt: Vec<Statement>,
  main_mod: Module,
  imports: ImportSection,
  data: DataSection,
  memory: MemorySection,
  types: TypeSection,
  functions: FunctionSection,
  exports: ExportSection,
  codes: CodeSection,
  literal_table: Vec<String>,
  fn_map: HashMap<String, usize>,
}

impl Codegen {
  const PRINT: u32 = 0;
  const PRINTLN: u32 = 1;

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
    }
  }

  fn finish(self) -> Vec<u8> {
    self.main_mod.finish().to_vec()
  }

  pub fn generate(mut self) -> Vec<u8> {
    self
      .imports
      .import("std", Some("print"), EntityType::Function(0));
    self
      .imports
      .import("std", Some("println"), EntityType::Function(0));

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
    self.fn_map.insert("print".into(), 0);
    self.fn_map.insert("println".into(), 1);

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
    for statement in &self.stmt {
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
          let mut func = Function::new(locals);
          for stmt in statements {
            match stmt {
              Statement::Terminate => {
                // TODO: actually do something with this
              }
              Statement::Print(literal) => {
                let offset = {
                  let mut offset = 0;
                  for lit in &self.literal_table {
                    offset += lit.len();
                  }
                  if offset > 0 {
                    offset += 1;
                  }
                  offset as i32
                };
                self
                  .data
                  .active(0, Instruction::I32Const(offset), literal.as_str().bytes());
                func.instruction(Instruction::I32Const(offset));
                func.instruction(Instruction::I32Const(offset + literal.len() as i32));
                func.instruction(Instruction::Call(Self::PRINT));
                self.literal_table.push(literal.as_str().into());
              }
              Statement::PrintLn(literal) => {
                let offset = {
                  let mut offset = 0;
                  for lit in &self.literal_table {
                    offset += lit.len();
                  }
                  if offset > 0 {
                    offset += 1;
                  }
                  offset as i32
                };
                self
                  .data
                  .active(0, Instruction::I32Const(offset), literal.as_str().bytes());
                func.instruction(Instruction::I32Const(offset));
                func.instruction(Instruction::I32Const(offset + literal.len() as i32));
                func.instruction(Instruction::Call(Self::PRINTLN));
                self.literal_table.push(literal.as_str().into());
              }
              Statement::FnCall { name, .. } => {
                func.instruction(Instruction::Call(
                  *self.fn_map.get(name.as_str()).unwrap() as u32
                ));
              }
              Statement::StateDefn { .. } => panic!("Cannot define states inside a state"),
            }
          }
          func.instruction(Instruction::End);

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
