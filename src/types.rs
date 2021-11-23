use crate::{Print, PrintLn};

#[derive(Debug, Clone)]
pub enum Statement {
  StateDefn {
    end: bool,
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
