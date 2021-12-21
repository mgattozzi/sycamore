use crate::{
  codegen::{Codegen, Generate},
  context::SycContext,
  types::StrLit,
};
use std::error::Error;
use wasm_encoder::*;
use wasmtime::Linker;

pub fn wasi_linker(linker: &mut Linker<SycContext>) -> Result<(), Box<dyn Error>> {
  wasmtime_wasi::add_to_linker(linker, |state: &mut SycContext| {
    state.wasi.as_mut().unwrap()
  })?;
  Ok(())
}

#[derive(Debug, Clone)]
pub enum Wasi {
  Print(StrLit),
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
        let offset = {
          let mut offset = 0;
          for lit in &codegen.literal_table {
            offset += lit.len();
          }
          if offset > 0 {
            offset += 5;
          }
          offset as i32
        };
        codegen
          .data
          .active(0, &Instruction::I32Const(offset), literal.as_str().bytes());

        const MAX_MEM: i32 = 65536;
        let ptr_base_loc = MAX_MEM - 16;
        let str_len_loc = MAX_MEM - 12;

        codegen.instruction(Instruction::I32Const(ptr_base_loc));
        codegen.instruction(Instruction::I32Const(offset));
        codegen.instruction(Instruction::I32Store(MemArg {
          memory_index: 0,
          align: 0,
          offset: 0,
        }));

        codegen.instruction(Instruction::I32Const(str_len_loc));
        codegen.instruction(Instruction::I32Const(literal.len() as i32));
        codegen.instruction(Instruction::I32Store(MemArg {
          memory_index: 0,
          align: 0,
          offset: 0,
        }));

        if self.is_println() {
          codegen.instruction(Instruction::I32Const(ptr_base_loc + 8));
          codegen.instruction(Instruction::I32Const(5));
          codegen.instruction(Instruction::I32Store(MemArg {
            memory_index: 0,
            align: 0,
            offset: 0,
          }));

          codegen.instruction(Instruction::I32Const(str_len_loc + 8));
          codegen.instruction(Instruction::I32Const(1));
          codegen.instruction(Instruction::I32Store(MemArg {
            memory_index: 0,
            align: 0,
            offset: 0,
          }));
        }

        // Set write to stdout
        codegen.instruction(Instruction::I32Const(1));
        // Pointer to array of iov
        codegen.instruction(Instruction::I32Const(ptr_base_loc));
        // Number of strings written
        if self.is_println() {
          codegen.instruction(Instruction::I32Const(2));
        } else {
          codegen.instruction(Instruction::I32Const(1));
        }
        // Where to store the number of bytes written
        codegen.instruction(Instruction::I32Const(0));
        // Call `fd_write`
        codegen.instruction(Instruction::Call(0));
        // Drop number of bytes written
        codegen.instruction(Instruction::Drop);

        codegen.literal_table.push(literal.as_str().into());
      }
    }
  }
}
