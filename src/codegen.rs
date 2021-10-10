use crate::parser::*;
use std::collections::HashMap;
use wasm_encoder::*;

pub struct Codegen {
  stmt: Vec<Statement>,
  main_mod: Module,
}

impl Codegen {
  const PRINT: u32 = 0;

  pub fn new(stmt: Vec<Statement>) -> Self {
    Self {
      stmt,
      main_mod: Module::new(),
    }
  }

  fn finish(self) -> Vec<u8> {
    self.main_mod.finish().to_vec()
  }

  pub fn generate(mut self) -> Vec<u8> {
    let mut imports = ImportSection::new();
    imports.import("std", Some("print"), EntityType::Function(0));

    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
      minimum: 1,
      maximum: None,
      memory64: false,
    });
    let mut types = TypeSection::new();
    // print function type
    types.function(vec![ValType::I32, ValType::I32], Vec::new());
    // void function type
    types.function(Vec::new(), Vec::new());

    let mut functions = FunctionSection::new();
    let mut exports = ExportSection::new();
    exports.export("main_memory", Export::Memory(0));
    let mut codes = CodeSection::new();
    let mut data = DataSection::new();
    let mut literal_table: Vec<String> = Vec::new();
    let mut fn_map: HashMap<String, usize> = HashMap::new();
    fn_map.insert("print".into(), 0);

    for fn_name in self.stmt.iter().filter_map(|a| {
      if let Statement::StateDefn { name, .. } = a {
        Some(name)
      } else {
        None
      }
    }) {
      fn_map.insert(fn_name.as_str().into(), fn_map.len());
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
            functions.function(void_function_index);
            exports.export("main", Export::Function(1));
          } else {
            let void_function_index = 1;
            functions.function(void_function_index);
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
                  for lit in &literal_table {
                    offset += lit.len();
                  }
                  if offset > 0 {
                    offset += 1;
                  }
                  offset as i32
                };
                data.active(0, Instruction::I32Const(offset), literal.as_str().bytes());
                func.instruction(Instruction::I32Const(offset));
                func.instruction(Instruction::I32Const(offset + literal.len() as i32));
                func.instruction(Instruction::Call(Self::PRINT));
                literal_table.push(literal.as_str().into());
              }
              Statement::FnCall { name, .. } => {
                func.instruction(Instruction::Call(*fn_map.get(name.as_str()).unwrap() as u32));
              }
              Statement::StateDefn { .. } => panic!("Cannot define states inside a state"),
            }
          }
          func.instruction(Instruction::End);

          codes.function(&func);
        }
        _ => panic!("Invalid only StateDefn are allowed"),
      }
    }

    // Set the sections in the right order
    self.main_mod.section(&types);
    self.main_mod.section(&imports);
    self.main_mod.section(&functions);
    self.main_mod.section(&memory);
    self.main_mod.section(&exports);
    self.main_mod.section(&codes);
    self.main_mod.section(&data);
    // Create and validate
    let wasm = self.finish();
    println!("---------------- Codegen WAT Output ----------------");
    println!("{}", wabt::wasm2wat(&wasm).unwrap_or(String::new()));

    if let Err(e) = wasmparser::validate(&wasm) {
      panic!("{}", e);
    }

    wasm
  }
}
