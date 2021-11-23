//use ariadne::{Label, Report, ReportKind, Source};
// Report::build(ReportKind::Error, (), 34)
//     .with_message("Incompatible types")
//     .with_label(Label::new(32..33).with_message("This is of type Nat"))
//     .with_label(Label::new(42..45).with_message("This is of type Str"))
//     .finish()
//     .print(Source::from(include_str!("../examples/sample.tao")))
//     .unwrap();
use crate::{Ident, Print, PrintLn, Statement, StrLit, Type};
use logos::{Lexer, Logos};

pub struct SycParser<'lex> {
  _input: &'lex str,
  lex: Lexer<'lex, Token>,
}

impl<'lex> SycParser<'lex> {
  pub fn new(_input: &'lex str) -> Self {
    let lex = Token::lexer(_input);
    Self { _input, lex }
  }

  pub fn expect(&mut self, t: Token, err: &str) {
    if self.next() != t {
      panic!("{}", err);
    }
  }

  pub fn next(&mut self) -> Token {
    match self.lex.next() {
      Some(t) => t,
      None => panic!("Hit EOF"),
    }
  }

  pub fn ident(&mut self) -> Ident {
    self.expect(Token::Identifier, "No ident token");
    self.mk_ident()
  }

  pub fn mk_ident(&mut self) -> Ident {
    Ident::new(self.lex.slice())
  }

  pub fn string_literal(&mut self) -> StrLit {
    self.expect(Token::StringLiteral, "No ident token");
    self.mk_str_lit()
  }

  pub fn mk_str_lit(&mut self) -> StrLit {
    let slice = self.lex.slice();
    // Get rid of the quotes here
    StrLit::new(&slice[1..slice.len() - 1])
  }

  pub fn next_opt(&mut self) -> Option<Token> {
    self.lex.next()
  }

  pub fn parse_args(&mut self) -> Vec<Type> {
    self.expect(Token::LParen, "No LParen token for args");
    let args = Vec::new();
    match self.next() {
      Token::RParen => return args,
      _ => unimplemented!("Args"),
    }
  }
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
            block.push(Statement::PrintLn(PrintLn::new(str_lit)));
          } else if ident.as_str() == "print" {
            self.expect(Token::LParen, "No LParen for print statement");
            let str_lit = self.string_literal();
            self.expect(Token::RParen, "No RParen for print statement");
            self.expect(Token::SemiColon, "No semicolon for print statement");
            block.push(Statement::Print(Print::new(str_lit)));
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

  pub fn parse(mut self) -> Vec<Statement> {
    let mut statements = Vec::new();
    loop {
      match self.next_opt() {
        Some(Token::End) => {
          self.expect(Token::State, "No state token after end");
          let state_defn = Statement::StateDefn {
            end: true,
            name: self.ident(),
            input: self.parse_args(),
            statements: self.parse_block(),
          };
          statements.push(state_defn);
        }
        Some(Token::State) => {
          let state_defn = Statement::StateDefn {
            end: false,
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

#[derive(Logos, Debug, PartialEq, Eq)]
pub enum Token {
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
  #[token("end")]
  End,
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

  #[regex("[a-zA-Z$_][a-zA-Z0-9$_]*")]
  Identifier,

  #[regex(r"[ \t\n\f]+", logos::skip)]
  Whitespace,

  #[error]
  Error,
}
