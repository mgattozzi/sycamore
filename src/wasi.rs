use crate::{
  codegen::{Codegen, Generate, MAX_MEM, RESULT_IDX},
  context::SycContext,
  types::StrLit,
};
use std::error::Error;
use wasm_encoder::*;
use wasmtime::Linker;

/// Put the wasi functions to link inside the [`wasmtime::Linker`]
pub fn wasi_linker(linker: &mut Linker<SycContext>) -> Result<(), Box<dyn Error>> {
  wasmtime_wasi::add_to_linker(linker, |state: &mut SycContext| {
    state.wasi.as_mut().unwrap()
  })?;
  Ok(())
}

#[derive(Debug, Clone)]
/// Enum of types of Input and Output that `sycamore` can do
pub enum Wasi {
  /// Print a given string literal
  Print(StrLit),
  /// Print a given string literal with a newline
  Println(StrLit),
}

impl Wasi {
  fn is_println(&self) -> bool {
    match self {
      Wasi::Println(_) => true,
      _ => false,
    }
  }
}

impl Generate for Wasi {
  fn generate(&self, codegen: &mut Codegen) {
    match self {
      Wasi::Print(literal) | Wasi::Println(literal) => {
        // Find where the string should be written into in memory
        let offset = {
          let mut offset = 0;
          for lit in &codegen.literal_table {
            offset += lit.len();
          }
          offset as i32
        };

        // Write the data into memory
        codegen
          .data
          .active(0, &Instruction::I32Const(offset), literal.as_str().bytes());

        // Setup pointers to the data to be printed out
        let mut io_vec = Vec::new();
        io_vec.push(IoVecItem::new(offset, literal.len() as i32));

        // If we are calling println point to the newline character in the
        // binary
        if self.is_println() {
          io_vec.push(IoVecItem::new(0, 1));
        }

        // Create the assmbly for the write to stdout
        codegen.fd_write(STDOUT, io_vec);

        // Push the literal into the codegen table so we can keep track of
        // things
        codegen.literal_table.push(literal.as_str().into());
      }
    }
  }
}

/// File Descriptor for the location of standard out
const STDOUT: i32 = 1;

pub trait WasiFnHelpers {
  fn write_io_vec(&mut self, io_vec: Vec<IoVecItem>) -> i32;
  fn wasi_imports(&mut self);
}

pub trait WasiFns {
  fn fd_write(&mut self, fd: i32, io_vec: Vec<IoVecItem>);
}

impl WasiFns for Codegen {
  /// Create instructions to write an `iov` to a given file descriptor
  fn fd_write(&mut self, fd: i32, io_vec: Vec<IoVecItem>) {
    let num_strs = io_vec.len() as i32;
    let io_vec_ptr = self.write_io_vec(io_vec);

    // Set write to given fd
    self.instruction(Instruction::I32Const(fd));
    // Pointer to array of iov
    self.instruction(Instruction::I32Const(io_vec_ptr));
    // Number of strings written
    self.instruction(Instruction::I32Const(num_strs));
    // Where to store the number of bytes written
    self.instruction(Instruction::I32Const(RESULT_IDX));
    // Call `fd_write`
    self.instruction(Instruction::Call(FD_WRITE));
    // Drop number of bytes written
    self.instruction(Instruction::Drop);
  }
}

// WASI Function Number Constants
const FD_WRITE: u32 = 0;

impl WasiFnHelpers for Codegen {
  /// Create instructions to write an iov into memory when executing a program
  fn write_io_vec(&mut self, io_vec: Vec<IoVecItem>) -> i32 {
    // Find the amount of bytes needed plus a little extra space for
    // the result
    let item_len_plus_result = ((io_vec.len() * 8) + 4) as i32;
    let ptr = MAX_MEM - item_len_plus_result;
    let mut current_idx = ptr;

    // Write code to store each item from the vec
    for IoVecItem { offset, len } in io_vec {
      self.instruction(Instruction::I32Const(current_idx));
      self.instruction(Instruction::I32Const(offset));
      self.instruction(Instruction::I32Store(MemArg {
        memory_index: 0,
        align: 0,
        offset: 0,
      }));
      current_idx += 4;
      self.instruction(Instruction::I32Const(current_idx));
      self.instruction(Instruction::I32Const(len));
      self.instruction(Instruction::I32Store(MemArg {
        memory_index: 0,
        align: 0,
        offset: 0,
      }));
      current_idx += 4;
    }

    // Return the location of the vec in memory
    ptr
  }

  /// Import all of the WASI functions for a `sycamore` program
  fn wasi_imports(&mut self) {
    self.types.function(
      vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
      vec![ValType::I32],
    );
    self.fn_map.insert("fd_write".into(), FD_WRITE);
    self.imports.import(
      "wasi_unstable",
      Some("fd_write"),
      EntityType::Function(FD_WRITE),
    );
  }
}

/// Struct for an entry in an `iov` in WASI. It contains an offset to an item
/// and the length of said item, usually used for printing strings.
pub struct IoVecItem {
  offset: i32,
  len: i32,
}

impl IoVecItem {
  /// Create a new `IoVecItem`
  pub fn new(offset: i32, len: i32) -> Self {
    Self { offset, len }
  }
}
