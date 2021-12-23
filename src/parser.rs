use crate::{
  types::{Ident, Statement, StrLit, SycValue, Type},
  wasi::Wasi,
};
use logos::{Logos, SpannedIter};
use std::{iter::Peekable, ops::Range};

/// A parser for a sycamore program. Given an input it will generate a
/// `Vec<Statement>` for use in a `Codegen` to create the output of the program
pub struct SycParser<'lex> {
  input: &'lex str,
  lex: Peekable<SpannedIter<'lex, Token>>,
  current: Option<(Token, Range<usize>)>,
}

impl<'lex> SycParser<'lex> {
  /// Create a new `SycParser`
  pub fn new(input: &'lex str) -> Self {
    let lex = Token::lexer(input).spanned().peekable();
    Self {
      input,
      lex,
      current: None,
    }
  }
  /// Get the slice of the `&str` for the current token
  pub fn slice(&self) -> &'lex str {
    &self.input[self.current.as_ref().unwrap().1.clone()]
  }

  /// Assume we get the next token and consume it. Panic with the given error
  /// string if this is not the case.
  pub fn expect(&mut self, t: Token, err: &str) {
    if self.next() != t {
      panic!("{}", err);
    }
  }

  /// Get the next `Token` to operate on
  pub fn next(&mut self) -> Token {
    match self.lex.next() {
      Some((t, r)) => {
        self.current = Some((t, r));
        t
      }
      None => panic!("Hit EOF"),
    }
  }

  /// Check if the next `Token` is equal to the given `Token`
  pub fn peek(&mut self, tok: Token) -> bool {
    self.lex.peek().map(|(t, _)| t) == Some(&tok)
  }

  /// Create an `Ident`
  pub fn ident(&mut self) -> Ident {
    self.expect(Token::Identifier, "No ident token");
    self.mk_ident()
  }

  /// Make an `Ident` from the given slice
  pub fn mk_ident(&mut self) -> Ident {
    Ident::new(self.slice())
  }

  /// Make a number from the given token
  pub fn mk_number(&mut self) -> SycValue {
    SycValue::I32(self.slice().parse::<i32>().expect("An i32 number"))
  }

  /// Create a `StrLit`
  pub fn string_literal(&mut self) -> StrLit {
    self.expect(Token::StringLiteral, "No ident token");
    self.mk_str_lit()
  }

  /// Make a `StrLit` from the given slice
  pub fn mk_str_lit(&mut self) -> StrLit {
    let slice = self.slice();
    // Get rid of the quotes here
    StrLit::new(&slice[1..slice.len() - 1])
  }

  /// Get the next token if it exists
  pub fn next_opt(&mut self) -> Option<Token> {
    self.current = self.lex.next();
    self.current.as_ref().map(|c| c.0)
  }

  /// Parse args for a function call
  pub fn parse_args(&mut self) -> Vec<Type> {
    self.expect(Token::LParen, "No LParen token for args");
    let args = Vec::new();
    match self.next() {
      Token::RParen => return args,
      _ => unimplemented!("Args"),
    }
  }
  /// Parse a block of statements
  pub fn parse_block(&mut self) -> Vec<Statement> {
    let mut block = Vec::new();
    self.expect(Token::LCurly, "No LCurly token for block");
    loop {
      match self.next_opt() {
        None => panic!("Hit end of file parsing block"),
        Some(Token::Identifier) => {
          let ident = self.mk_ident();

          if ident.as_str() == "println" {
            self.expect(Token::LParen, "No LParen for println statement");
            let str_lit = self.string_literal();
            self.expect(Token::RParen, "No RParen for println statement");
            self.expect(Token::SemiColon, "No semicolon for println statement");
            block.push(Statement::Wasi(Wasi::Println(str_lit)));
          } else if ident.as_str() == "print" {
            self.expect(Token::LParen, "No LParen for print statement");
            let str_lit = self.string_literal();
            self.expect(Token::RParen, "No RParen for print statement");
            self.expect(Token::SemiColon, "No semicolon for print statement");
            block.push(Statement::Wasi(Wasi::Print(str_lit)));
          } else if self.peek(Token::Assign) {
            self.next();
            self.next();
            let value = self.mk_number();
            self.expect(Token::SemiColon, "No semicolon for assignment statement");
            block.push(Statement::Assignment { name: ident, value });
          } else {
            self.expect(Token::LParen, "No LParen for fn statement");
            self.expect(Token::RParen, "No funcs with more than 0 args for now");
            self.expect(Token::SemiColon, "No semicolon for fn statement");
            block.push(Statement::FnCall {
              name: ident,
              input: Vec::new(),
            });
          }
        }
        Some(Token::Terminate) => {
          self.expect(Token::SemiColon, "No semicolon for print statement");
          block.push(Statement::Terminate);
        }
        Some(Token::RCurly) => break,
        Some(_) => unimplemented!("Assignment etc."),
      }
    }

    block
  }

  /// Parse the input into the final output
  pub fn parse(mut self) -> Vec<Statement> {
    let mut statements = Vec::new();
    loop {
      match self.next_opt() {
        Some(Token::Terminating) => {
          self.expect(Token::State, "No state token after end");
          let state_defn = Statement::StateDefn {
            terminating: true,
            name: self.ident(),
            input: self.parse_args(),
            statements: self.parse_block(),
          };
          statements.push(state_defn);
        }
        Some(Token::State) => {
          let state_defn = Statement::StateDefn {
            terminating: false,
            name: self.ident(),
            input: self.parse_args(),
            statements: self.parse_block(),
          };
          statements.push(state_defn);
        }
        None => break,
        _ => (),
      }
    }

    statements
  }
}

/// All the valid tokens in a sycamore program
#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
  // Assignment
  #[token("<-")]
  Assign,

  // Comparators
  #[token("and")]
  And,
  #[token("or")]
  Or,
  #[token("equals")]
  Equals,

  // Control flow
  #[token("goto")]
  GoTo,
  #[token("terminate")]
  Terminate,
  #[token("unreachable")]
  Unreachable,
  #[token("if")]
  If,
  #[token("else")]
  Else,

  // State
  #[token("terminating")]
  Terminating,
  #[token("state")]
  State,

  #[token("(")]
  LParen,
  #[token(")")]
  RParen,
  #[token("{")]
  LCurly,
  #[token("}")]
  RCurly,
  #[token(";")]
  SemiColon,

  #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
  StringLiteral,

  #[regex("[0-9]*")]
  Number,

  #[regex("[a-zA-Z$-][a-zA-Z0-9$-]*")]
  Identifier,

  #[regex(r"[ \t\n\f]+", logos::skip)]
  Whitespace,

  #[error]
  Error,
}
