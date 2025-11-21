use crate::cpu::{CPU, RegisterName};
use crate::memory::Memory;
use crate::decoder::{Instruction, Opcode, Operand};

/// Get the length of a JMP instruction based on opcode and ModR/M byte if present
/// 
/// Returns the total length in bytes:
/// - Short JMP (EB): 2 bytes (opcode + 8-bit displacement)
/// - Near JMP (E9): 5 bytes (opcode + 32-bit displacement)
pub fn get_jmp_length(opcode: u8, modrm_byte: Option<u8>) -> u8 {
    match opcode {
        0xEB => 2,  // Short JMP: opcode + 8-bit displacement
        0xE9 => 5,  // Near JMP: opcode + 32-bit displacement
        0xFF => {
            // For FF opcode, length depends on ModR/M mode bits
            if let Some(modrm) = modrm_byte {
                let mode = (modrm >> 6) & 0b11;
                match mode {
                    0b00 => 2, // Register indirect or memory, no displacement
                    0b01 => 3, // Memory + 8-bit displacement
                    0b10 => 6, // Memory + 32-bit displacement
                    0b11 => 2, // Register direct
                    _ => 0,
                }
            } else {
                2  // Default to basic ModR/M length 
            }
        }
        _ => 0,
    }
}

/// Execute a JMP instruction
/// 
/// This function executes the x86 JMP instruction in its various forms:
/// - Short JMP (EB): 8-bit relative displacement
/// - Near JMP (E9): 32-bit relative displacement
/// - Indirect JMP (FF /4): Through register or memory
/// 
/// # Arguments
/// * `cpu` - Mutable reference to CPU state
/// * `memory` - Mutable reference to memory system
/// * `instruction` - The decoded instruction
/// 
/// # Returns
/// * `Ok(())` - Jump executed successfully
/// * `Err(String)` - Error during execution
pub(crate) fn execute(cpu: &mut CPU, memory: &mut Memory, instruction: &Instruction) -> Result<(), String> {
    if instruction.opcode != Opcode::JMP {
        return Err("Invalid opcode for JMP instruction".to_string());
    }

    if let Some(dest) = &instruction.dest {
        match dest {
            Operand::Immediate(displacement) => {
                match instruction.length {
                    2 => {
                        // Short JMP (8-bit displacement)
                        let displacement = *displacement as i8;
                        let new_ip = ((cpu.registers.eip as i16) + 2 + (displacement as i16)) as u32;
                        cpu.registers.eip = new_ip;
                    },
                    5 => {
                        // Near JMP (32-bit displacement)
                        let displacement = *displacement as i32;
                        let new_ip = ((cpu.registers.eip as i32) + 5 + displacement) as u32;
                        cpu.registers.eip = new_ip;
                    },
                    _ => return Err("Invalid JMP instruction length".to_string()),
                }
            },
            Operand::Register(reg) => {
                // Indirect JMP through register
                let target = match reg {
                    RegisterName::EAX => cpu.registers.eax,
                    RegisterName::ECX => cpu.registers.ecx,
                    RegisterName::EDX => cpu.registers.edx,
                    RegisterName::EBX => cpu.registers.ebx,
                    RegisterName::ESP => cpu.registers.esp,
                    RegisterName::EBP => cpu.registers.ebp,
                    RegisterName::ESI => cpu.registers.esi,
                    RegisterName::EDI => cpu.registers.edi,
                    RegisterName::EIP => cpu.registers.eip,
                };
                cpu.registers.eip = target;
            },
            Operand::Memory(addr) => {
                // Indirect JMP through memory
                let target = memory.read_u32(*addr).map_err(|e| e.to_string())?;
                cpu.registers.eip = target;
            },
        }
        Ok(())
    } else {
        Err("Missing destination operand for JMP instruction".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::CPU;
    use crate::memory::Memory;
    use crate::decoder::{Instruction, Opcode, Operand};
    use crate::cpu::RegisterName;

    #[test]
    fn test_short_jmp_forward() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        cpu.registers.eip = 0x1000;
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: Some(Operand::Immediate(5)),
            src: None,
            length: 2,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eip, 0x1007);  // 0x1000 + 2 + 5
    }

    #[test]
    fn test_short_jmp_backward() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        cpu.registers.eip = 0x1000;
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: Some(Operand::Immediate(-5i8 as u32)),
            src: None,
            length: 2,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eip, 0x0FFD);  // 0x1000 + 2 - 5
    }

    #[test]
    fn test_near_jmp_forward() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        cpu.registers.eip = 0x1000;
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: Some(Operand::Immediate(0x100)),
            src: None,
            length: 5,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eip, 0x1105);  // 0x1000 + 5 + 0x100
    }

    #[test]
    fn test_indirect_jmp_register() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        cpu.registers.eip = 0x1000;
        cpu.registers.eax = 0x2000;
        
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: Some(Operand::Register(RegisterName::EAX)),
            src: None,
            length: 2,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eip, 0x2000);
    }

    #[test]
    fn test_indirect_jmp_memory() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        cpu.registers.eip = 0x1000;
        memory.write_u32(0x2000, 0x3000).unwrap();
        
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: Some(Operand::Memory(0x2000)),
            src: None,
            length: 2,
        };
        
        execute(&mut cpu, &mut memory, &instruction).unwrap();
        assert_eq!(cpu.registers.eip, 0x3000);
    }

    #[test]
    fn test_invalid_opcode() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        let instruction = Instruction {
            opcode: Opcode::PUSH,  // Wrong opcode
            dest: Some(Operand::Immediate(5)),
            src: None,
            length: 2,
        };
        
        let result = execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_destination() {
        let mut cpu = CPU::new();
        let mut memory = Memory::default();
        
        let instruction = Instruction {
            opcode: Opcode::JMP,
            dest: None,  // Missing destination
            src: None,
            length: 2,
        };
        
        let result = execute(&mut cpu, &mut memory, &instruction);
        assert!(result.is_err());
    }
}
