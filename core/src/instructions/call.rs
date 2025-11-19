//! CALL Instruction Implementation
//!
//! What this does:
//! 1. Calculates where to return after the call
//! 2. Pushes that address onto the stack
//! 3. Jumps to the new function location using the rel32 offset

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::decoder::{Instruction, Operand};
use super::push::ExecutionError; // using same error type as PUSH

/// Runs a CALL rel32 instruction
pub fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), ExecutionError> {
    // figure out where the CPU should come back after this call
    let return_eip = cpu.registers.eip.wrapping_add(instruction.length as u32);

    // make sure we don’t underflow the stack
    if cpu.registers.esp < 4 {
        return Err(ExecutionError::StackOverflow);
    }

    // push that return address onto the stack
    let new_esp = memory.push_u32(cpu.registers.esp, return_eip)?;
    cpu.registers.esp = new_esp;

    // get the relative jump offset (rel32) from the instruction
    let disp_i32 = extract_rel32(instruction).ok_or(ExecutionError::InvalidOperand)?;

    // move EIP to the new function address (EIP = return_eip + rel32)
    cpu.registers.eip = return_eip.wrapping_add(disp_i32 as u32);

    Ok(())
}

/// grabs the rel32 offset from the instruction (either dest or src)
fn extract_rel32(instr: &Instruction) -> Option<i32> {
    // sometimes it's in dest, sometimes in src — we check both
    let from_op = instr.dest.as_ref().or(instr.src.as_ref());
    match from_op {
        Some(Operand::Immediate(val)) => Some(*val as i32),
        _ => None,
    }
}

/// helper for testing CALL directly without a full decode
pub fn call_rel32(cpu: &mut CPU, mem: &mut Memory, disp: i32, instr_len: u32) {
    // compute where to come back after call
    let return_eip = cpu.registers.eip.wrapping_add(instr_len);

    // push return address to the stack
    let new_esp = mem.push_u32(cpu.registers.esp, return_eip).unwrap();
    cpu.registers.esp = new_esp;

    // jump to the target (new function)
    cpu.registers.eip = return_eip.wrapping_add(disp as u32);
}


#[cfg(test)]
mod tests {
    use super::{call_rel32, execute};
    use crate::cpu::CPU;
    use crate::memory::Memory;
    use crate::decoder::{Instruction, Opcode, Operand};

    // --- Direct helper tests (call_rel32) ---

    #[test]
    fn call_rel32_forward_jump_pushes_return_and_updates_eip() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new(0x1000000);

        cpu.registers.eip = 0x1000;
        cpu.registers.esp = 0x2000;

        // simulate CALL with +0x50 displacement and 5-byte length (E8 + imm32)
        call_rel32(&mut cpu, &mut mem, 0x50, 5);

        let expected_return = 0x1000 + 5;
        assert_eq!(cpu.registers.esp, 0x1FFC, "ESP should drop by 4");
        assert_eq!(mem.read_u32(cpu.registers.esp).unwrap(), expected_return, "Top of stack holds return EIP");
        assert_eq!(cpu.registers.eip, expected_return + 0x50, "EIP should point to target");
    }

    #[test]
    fn call_rel32_negative_jump_pushes_return_and_updates_eip() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new(0x1000000);

        cpu.registers.eip = 0x2000;
        cpu.registers.esp = 0x3000;

        // simulate CALL with -0x30 displacement
        call_rel32(&mut cpu, &mut mem, -0x30, 5);

        let expected_return = 0x2000 + 5;
        assert_eq!(cpu.registers.esp, 0x2FFC, "ESP should drop by 4");
        assert_eq!(mem.read_u32(cpu.registers.esp).unwrap(), expected_return, "Return EIP should be pushed");
        assert_eq!(
            cpu.registers.eip,
            expected_return.wrapping_add(-0x30_i32 as u32),
            "EIP should wrap correctly for negative disp"
        );
    }

    // --- Execute() tests (full instruction path) ---

    #[test]
    fn execute_call_rel32_forward() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new(0x1000000);

        cpu.registers.eip = 0x1000;
        cpu.registers.esp = 0x8000;

        // Build a decoded CALL rel32 instruction (length 5: E8 + imm32)
        let instr = Instruction {
            opcode: Opcode::CALL,
            dest: None,
            src: Some(Operand::Immediate(0x40)), // +0x40
            length: 5,
        };

        execute(&mut cpu, &mut mem, &instr).unwrap();

        let ret = 0x1000 + 5;
        assert_eq!(cpu.registers.esp, 0x7FFC);
        assert_eq!(mem.read_u32(0x7FFC).unwrap(), ret);
        assert_eq!(cpu.registers.eip, ret + 0x40);
    }

    #[test]
    fn execute_call_rel32_negative() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new(0x1000000);

        cpu.registers.eip = 0x4000;
        cpu.registers.esp = 0x5000;

        // -0x24 displacement in two's complement fits via casting in execute()
        let instr = Instruction {
            opcode: Opcode::CALL,
            dest: None,
            src: Some(Operand::Immediate((-0x24i32) as u32)),
            length: 5,
        };

        execute(&mut cpu, &mut mem, &instr).unwrap();

        let ret = 0x4000 + 5;
        assert_eq!(cpu.registers.esp, 0x4FFC);
        assert_eq!(mem.read_u32(0x4FFC).unwrap(), ret);
        assert_eq!(cpu.registers.eip, ret.wrapping_add(-0x24i32 as u32));
    }

    #[test]
    fn execute_call_stack_underflow_is_error() {
        let mut cpu = CPU::new();
        let mut mem = Memory::new(0x100000);

        cpu.registers.eip = 0x10;
        cpu.registers.esp = 2; // < 4 triggers the same guard as PUSH

        let instr = Instruction {
            opcode: Opcode::CALL,
            dest: None,
            src: Some(Operand::Immediate(0x10)),
            length: 5,
        };

        assert!(super::execute(&mut cpu, &mut mem, &instr).is_err());
    }
}