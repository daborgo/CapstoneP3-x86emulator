//! RET Instruction
//! Pop 4 bytes from [ESP] into EIP; ESP += 4.

use std::fmt;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::Instruction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    StackUnderflow,

    MemoryError(crate::memory::MemoryError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::StackUnderflow => write!(f, "Stack underflow"),
            ExecutionError::MemoryError(err) => write!(f, "Memory error: {}", err),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<crate::memory::MemoryError> for ExecutionError {
    fn from(err: crate::memory::MemoryError) -> Self {
        ExecutionError::MemoryError(err)
    }
}

/// Execute a RET instruction.
///
/// Steps:
/// 1) read u32 at [ESP]  - return address
/// 2) ESP += 4           - pop
/// 3) EIP = return address (control transfer)
pub fn execute(cpu: &mut CPU, memory: &mut Memory, _instruction: &Instruction)
    -> Result<(), ExecutionError>
{
    let esp = cpu.registers.esp;

    // Pop return address from stack
    // Memory bounds checking is handled by read_u32
    let ret_addr = memory.read_u32(esp)?;
    cpu.registers.esp = esp.wrapping_add(4);
    cpu.registers.eip = ret_addr;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{Instruction, Opcode};

    #[test]
    fn ret_pops_and_jumps() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x0100_0000); // 16MB

        cpu.registers.esp = 0x00FF_0000;
        memory.write_u32(0x00FF_0000, 0x1234_5678).unwrap();

        let instr = Instruction { opcode: Opcode::RET, dest: None, src: None, length: 1 };

        execute(&mut cpu, &mut memory, &instr).unwrap();

        assert_eq!(cpu.registers.esp, 0x00FF_0004);
        assert_eq!(cpu.registers.eip, 0x1234_5678);
    }

    #[test]
    fn ret_underflow_errors() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);

        cpu.registers.esp = 0x0FFD; // only 3 bytes left -> not enough for u32
        let instr = Instruction { opcode: Opcode::RET, dest: None, src: None, length: 1 };

        let err = execute(&mut cpu, &mut memory, &instr).unwrap_err();
        assert!(matches!(err, ExecutionError::StackUnderflow));
    }
}
