use crate::{
  codegen::{Codegen, Generate},
  wasi::Wasi,
};
use std::collections::HashMap;
use wasm_encoder::*;

#[derive(Debug, Clone)]
pub enum Statement {
  Assignment {
    name: Ident,
    value: SycValue,
  },
  StateDefn {
    terminating: bool,
    name: Ident,
    input: Vec<Type>,
    statements: Vec<Statement>,
  },
  FnCall {
    name: Ident,
    input: Vec<Type>,
  },
  Wasi(Wasi),
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
              codegen.current_func.as_mut().map(|f| {
                let local = locals_map
                  .get(name.as_str())
                  .expect("locals_map was already populated");
                match value {
                  SycValue::I32(v) => {
                    f.instruction(&Instruction::I32Const(*v));
                  }
                }
                f.instruction(&Instruction::LocalSet(*local));
                f
              });
            }
            Statement::Terminate => {
              // TODO: actually do something with this
            }
            Statement::Wasi(wasi) => wasi.generate(codegen),
            Statement::FnCall { name, .. } => {
              codegen.current_func.as_mut().map(|f| {
                f.instruction(&Instruction::Call(
                  *codegen.fn_map.get(name.as_str()).unwrap() as u32,
                ));
                f
              });
            }
            Statement::StateDefn { .. } => panic!("Cannot define states inside a state"),
          }
        }
        codegen.current_func.as_mut().map(|f| {
          f.instruction(&Instruction::End);
          f
        });

        let func = codegen.current_func.take().unwrap();
        codegen.codes.function(&func);
      }
      _ => panic!("Invalid only StateDefn are allowed"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Type {
  I32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ident(String);

impl Ident {
  pub fn new(input: impl ToString) -> Self {
    Self(input.to_string())
  }
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StrLit(String);

impl StrLit {
  pub fn new(input: impl ToString) -> Self {
    Self(input.to_string())
  }
  pub fn as_str(&self) -> &str {
    &self.0
  }
  pub fn len(&self) -> usize {
    self.0.len()
  }
}

#[derive(Debug, Clone)]
pub enum SycValue {
  I32(i32),
}

impl SycValue {
  pub fn from_i32(input: i32) -> Self {
    Self::I32(input)
  }
  pub fn as_val_type(&self) -> ValType {
    match self {
      Self::I32(_) => ValType::I32,
    }
  }
}
