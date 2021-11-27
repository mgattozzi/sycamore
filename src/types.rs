use crate::{Codegen, Generate, Print, PrintLn};
use wasm_encoder::*;

#[derive(Debug, Clone)]
pub enum Statement {
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
  Print(Print),
  PrintLn(PrintLn),
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
            .export("main", Export::Function(function_num));
        } else {
          codegen.functions.function(function_num);
        }

        let locals = Vec::new();
        codegen.current_func = Some(Function::new(locals));
        for stmt in statements {
          match stmt {
            Statement::Terminate => {
              // TODO: actually do something with this
            }
            Statement::Print(print) => print.generate(codegen),
            Statement::PrintLn(println) => println.generate(codegen),
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
