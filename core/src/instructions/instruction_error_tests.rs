//! Test cases for instruction error checking

#[cfg(test)]
mod tests {
    // Import necessary modules and types
    use crate::cpu::{CPU, RegisterName};
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::instructions::{mov, add, sub};
    use crate::memory::Memory;

    #[test]
    fn test_mov_memory_access_error() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000); // Small memory for out-of-bounds
        cpu.registers.eip = 0x1000;
        // Try to move from an out-of-bounds memory address
        let instruction = Instruction {
            opcode: Opcode::MOV,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Memory(0x2000)), // Out of bounds
            length: 2,
        };
        let result = mov::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
        if let Err(mov::ExecutionError::MemoryError(err)) = result {
            assert!(format!("{}", err).contains("out of bounds"));
        } else {
            panic!("Expected memory error");
        }
    }

    #[test]
    fn test_mov_invalid_operand_error() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        // Both dest and src are None (invalid instruction)
        let instruction = Instruction {
            opcode: Opcode::MOV,
            dest: None,
            src: None,
            length: 1,
        };
        let result = mov::execute(&mut cpu, &mut memory, &instruction);
        assert!(matches!(result, Err(mov::ExecutionError::InvalidOperand)));
    }

    #[test]
    fn test_add_and_sub_combined() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eax = 10;
        cpu.registers.ebx = 5;
        // ADD EAX, EBX (EAX = 10 + 5 = 15)
        let add_inst = Instruction {
            opcode: Opcode::ADD,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Register(RegisterName::EBX)),
            length: 1,
        };
        add::add(&mut cpu, &mut memory, &add_inst).unwrap();
        assert_eq!(cpu.registers.eax, 15);
        // SUB EAX, Immediate 3 (EAX = 15 - 3 = 12)
        let sub_inst = Instruction {
            opcode: Opcode::SUB,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Immediate(3)),
            length: 1,
        };
        sub::execute(&mut cpu, &mut memory, &sub_inst).unwrap();
        assert_eq!(cpu.registers.eax, 12);
    }

    #[test]
    fn test_add_invalid_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        // ADD with invalid dest (immediate)
        let add_inst = Instruction {
            opcode: Opcode::ADD,
            dest: Some(Operand::Immediate(1)),
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        let result = add::add(&mut cpu, &mut memory, &add_inst);
        assert!(result.is_err());
    }

    #[test]
    fn test_sub_invalid_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        // SUB with invalid src (memory, which is not supported in this implementation)
        let sub_inst = Instruction {
            opcode: Opcode::SUB,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: Some(Operand::Memory(0x10)),
            length: 1,
        };
        let result = sub::execute(&mut cpu, &mut memory, &sub_inst);
        assert!(result.is_err());
    }

    // #[test]
    // fn test_cmp_invalid_operand() {
    //     let mut cpu = CPU::new();
    //     let mut memory = Memory::new(0x1000);
    //     // CMP with invalid dest (immediate)
    //     let instruction = Instruction {
    //         opcode: Opcode::CMP,
    //         dest: Some(Operand::Immediate(1)),
    //         src: Some(Operand::Register(RegisterName::EAX)),
    //         length: 1,
    //     };
    //     let result = crate::instructions::cmp::execute(&mut cpu, &mut memory, &instruction);
    //     assert!(result.is_err());
    // }

    #[test]
    fn test_push_stack_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.esp = 2; // Too small for push
        let instruction = Instruction {
            opcode: Opcode::PUSH,
            dest: None,
            src: Some(Operand::Register(RegisterName::EAX)),
            length: 1,
        };
        let result = crate::instructions::push::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }

    #[test]
    fn test_pop_invalid_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.esp = 0x1000;
        let instruction = Instruction {
            opcode: Opcode::POP,
            dest: Some(Operand::Immediate(1)), // Invalid dest
            src: None,
            length: 1,
        };
        let result = crate::instructions::pop::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_invalid_operand() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.eip = 0x1000;
        cpu.registers.esp = 0x1000;
        let instruction = Instruction {
            opcode: Opcode::CALL,
            dest: None,
            src: None,
            length: 5,
        };
        let result = crate::instructions::call::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }

    #[test]
    fn test_ret_stack_underflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        cpu.registers.esp = 0xFFFF_FFFF; // Out of bounds
        let instruction = Instruction {
            opcode: Opcode::RET,
            dest: None,
            src: None,
            length: 1,
        };
        let result = crate::instructions::ret::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }

    #[test]
    fn test_jmp_invalid_opcode() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(0x1000);
        let instruction = Instruction {
            opcode: Opcode::ADD, // Not JMP
            dest: Some(Operand::Immediate(1)),
            src: None,
            length: 2,
        };
        let result = crate::instructions::jmp::execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }
}
