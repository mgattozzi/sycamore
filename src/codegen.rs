use crate::{context::SycContext, types::Statement, wasi::WasiFnHelpers};
use std::collections::HashMap;
use wasm_encoder::*;

pub const MAX_MEM: i32 = 65536;
pub const RESULT_IDX: i32 = MAX_MEM - 4;

/// `Codegen` is the main driver in sycamore that wraps various sections of a
/// wasm file with various fields to keep track of things. The `Generate` trait
/// is passed the `Codegen` object that various parts of the process can use to
/// actually drive the process. When all the code is made a call to `finish`
/// will generate the final wasm binary.
pub struct Codegen {
  /// Boolean to determine if we print out debug info when generating code
  pub debug: bool,
  /// List of all `Statements` that the code will be generated from
  pub stmt: Vec<Statement>,
  /// The `Module` that will be output at the end of code generation
  pub main_mod: Module,
  /// WebAssembly Import Section
  pub imports: ImportSection,
  /// WebAssembly Data Section
  pub data: DataSection,
  /// WebAssembly Memory Section
  pub memory: MemorySection,
  /// All of the names of various items for the WebAssembly Name Section
  pub name: Name,
  /// WebAssembly Type Section
  pub types: TypeSection,
  /// WebAssembly Function Section
  pub functions: FunctionSection,
  /// WebAssembly Export Section
  pub exports: ExportSection,
  /// WebAssembly Code Section
  pub codes: CodeSection,
  /// Table of String Literals that we use while generating code
  pub literal_table: Vec<String>,
  /// Map of Function Name to Function Number in the binary file
  pub fn_map: HashMap<String, u32>,
  /// The current function we are operating on
  pub current_func: Option<Function>,
  pub ctx: SycContext,
}

/// A struct of all names for the WebAssembly Name Section
pub struct Name {
  pub function_names: NameMap,
  pub memory_names: NameMap,
  pub type_names: NameMap,
  pub local_names: IndirectNameMap,
}

impl Name {
  /// Create a new `Name`
  pub fn new() -> Self {
    Self {
      function_names: NameMap::new(),
      memory_names: NameMap::new(),
      type_names: NameMap::new(),
      local_names: IndirectNameMap::new(),
    }
  }

  /// Make the `NameSection` for the wasm
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
  /// Create a new `Codegen`
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
      literal_table: Vec::new(),
      fn_map: HashMap::new(),
      current_func: None,
      ctx: SycContext::new(),
    }
  }

  /// Write an instruction into the current function code is being generated
  /// for.
  ///
  /// # Panics
  /// This will panic if no function is currently being worked on
  pub fn instruction(&mut self, instruction: Instruction) {
    self
      .current_func
      .as_mut()
      .unwrap()
      .instruction(&instruction);
  }

  /// Turn all of the code into a wasm binary
  fn finish(self) -> Vec<u8> {
    self.main_mod.finish().to_vec()
  }

  /// Generate the code for the given statements and consume the `Codegen` in the
  /// process.
  pub fn generate(mut self) -> Vec<u8> {
    self.wasi_imports();
    self.memory.memory(MemoryType {
      minimum: 1,
      maximum: None,
      memory64: false,
    });

    self.name.memory_names.append(0, "memory");
    self.exports.export("memory", Export::Memory(0));

    // Setup the new line for printing with a newline
    self.literal_table.push("\n".into());
    self.data.active(0, &Instruction::I32Const(0), ['\n' as u8]);

    // Setup the function map and types after our import so that we can make
    // calls to them properly everywhere. Also create all of our string literals
    // before hand.
    for stmt in self.stmt.iter() {
      if let Statement::StateDefn { name, .. } = stmt {
        // All states are the void type for now until we deal with args
        self.types.function(Vec::new(), Vec::new());
        let name = name.as_str().to_string();
        let idx = self.fn_map.len() as u32;
        self.name.function_names.append(idx, &name);
        self.name.type_names.append(idx, &name);
        self.fn_map.insert(name, idx);
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
    self.main_mod.section(&CustomSection {
      name: "SycContext",
      data: &bincode::serialize(&self.ctx).unwrap(),
    });
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

    wasm
  }
}

/// The main driver trait for code generation. Define how code is generated for
/// a type and pass a `Codegen` type into it. This then gets called to generate
/// code for everything.
pub trait Generate {
  fn generate(&self, codegen: &mut Codegen);
}
