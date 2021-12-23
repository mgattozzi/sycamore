use crate::{
  codegen::{Codegen, Generate},
  wasi::Wasi,
};
use std::collections::HashMap;
use wasm_encoder::*;

/// A `Statement` is the top level item in a sycamore program. It can define
/// items like states or do things inside of them such as `Assignment` or
/// `FnCall`s. A program is a `Vec<Statement>` that gets passed to a `Codegen`
/// in order for code to be generated for it
#[derive(Debug, Clone)]
pub enum Statement {
  /// Assigns a value to an `Ident`
  Assignment { name: Ident, value: SycValue },
  /// Defines a state for the program
  StateDefn {
    terminating: bool,
    name: Ident,
    input: Vec<Type>,
    statements: Vec<Statement>,
  },
  /// Makes a function call for a program
  FnCall { name: Ident, input: Vec<Type> },
  /// Makes a WASI function call
  Wasi(Wasi),
  /// Terminates the program
  Terminate,
}

impl Generate for Statement {
  fn generate(&self, codegen: &mut Codegen) {
    match self {
      Statement::StateDefn {
        name,
        terminating,
        input,
        statements,
      } => {
        let function_num = *codegen.fn_map.get(name.as_str()).unwrap();

        if name.as_str() == "main" {
          if !terminating {
            panic!("Main must be labelled an end state");
          }
          if !input.is_empty() {
            panic!("Main must have no arguments");
          }
          codegen.functions.function(function_num);
          codegen
            .exports
            .export("_start", Export::Function(function_num));
        } else {
          codegen.functions.function(function_num);
        }

        let mut locals = Vec::new();
        let mut locals_map = HashMap::new();

        // Create all the locals to be declared in the function
        for stmt in statements {
          if let Statement::Assignment { value, name } = stmt {
            locals_map.insert(name.as_str(), locals_map.len() as u32);
            locals.push(value.as_val_type());
          }
        }
        codegen.current_func = Some(Function::new_with_locals_types(locals));

        for stmt in statements {
          match stmt {
            Statement::Assignment { name, value } => {
              let local = locals_map
                .get(name.as_str())
                .expect("locals_map was already populated");
              match value {
                SycValue::I32(v) => {
                  codegen.instruction(Instruction::I32Const(*v));
                }
              }
              codegen.instruction(Instruction::LocalSet(*local));
            }
            Statement::Terminate => {
              // TODO: actually do something with this
            }
            Statement::Wasi(wasi) => wasi.generate(codegen),
            Statement::FnCall { name, .. } => {
              codegen.instruction(Instruction::Call(
                *codegen.fn_map.get(name.as_str()).unwrap() as u32,
              ));
            }
            Statement::StateDefn { .. } => panic!("Cannot define states inside a state"),
          }
        }
        codegen.instruction(Instruction::End);

        let mut local_names = NameMap::new();
        for (name, idx) in locals_map.iter() {
          local_names.append(*idx, name);
        }
        codegen.name.local_names.append(function_num, &local_names);
        codegen
          .codes
          .function(&codegen.current_func.take().unwrap());
      }
      _ => panic!("Invalid only StateDefn are allowed"),
    }
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Type {
  I32,
}

/// An identifier for a state, variable, or something else
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ident(String);

impl Ident {
  /// Create a new `Ident`
  pub fn new(input: impl ToString) -> Self {
    Self(input.to_string())
  }
  /// Get an `&str` of the `Ident`
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// A string literal defined in the source code
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StrLit(String);

impl StrLit {
  /// Create a new `StrLit`
  pub fn new(input: impl ToString) -> Self {
    Self(input.to_string())
  }
  /// Get an `&str` of the `StrLit`
  pub fn as_str(&self) -> &str {
    &self.0
  }
  /// Get the length of the `StrLit`
  pub fn len(&self) -> usize {
    self.0.len()
  }
}

/// A sycamore program type and the value of said type
#[derive(Debug, Clone)]
pub enum SycValue {
  I32(i32),
}

impl SycValue {
  #[allow(dead_code)]
  pub fn from_i32(input: i32) -> Self {
    Self::I32(input)
  }

  pub fn as_val_type(&self) -> ValType {
    match self {
      Self::I32(_) => ValType::I32,
    }
  }
}
