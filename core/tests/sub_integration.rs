use core::instructions::sub::{sub8, sub16, sub32, CpuFlags, execute};
use core::cpu::{CPU, RegisterName};
use core::memory::Memory;
use core::decoder::{Instruction, Opcode, Operand};

// Unit tests for low-level sub helpers
#[test]
fn sub8_basic() {
    let (r, f) = sub8(CpuFlags::default(), 5, 3);
    assert_eq!(r, 2);
    assert!(!f.cf);
    assert!(!f.of);
    assert!(!f.zf);
    assert!(!f.sf);
}

#[test]
fn sub8_borrow_cf() {
    let (r, f) = sub8(CpuFlags::default(), 0x00, 0x01);
    assert_eq!(r, 0xFF);
    assert!(f.cf);
    assert!(!f.zf);
    assert!(f.sf);
}

#[test]
fn sub8_overflow() {
    let (r, f) = sub8(CpuFlags::default(), 0x80, 0x01);
    assert_eq!(r, 0x7F);
    assert!(f.of);
    assert!(!f.cf);
}

#[test]
fn sub16_zero() {
    let (r, f) = sub16(CpuFlags::default(), 0x1234, 0x1234);
    assert_eq!(r, 0);
    assert!(f.zf);
    assert!(!f.cf);
    assert!(!f.of);
    assert!(!f.sf);
}

#[test]
fn sub32_sign() {
    let (r, f) = sub32(CpuFlags::default(), 1, 2);
    assert_eq!(r, 0xFFFF_FFFF);
    assert!(f.sf);
    assert!(f.cf);
    assert!(!f.zf);
}

// Integration tests that exercise the public CPU/memory/instruction APIs
#[test]
fn test_execute_register_to_register() {
    let mut cpu = CPU::new();
    let mut memory = Memory::new(1024);

    cpu.registers.set(RegisterName::EAX, 0x1234_5678);
    cpu.registers.set(RegisterName::EBX, 0x1234_0000);

    let instruction = Instruction {
        opcode: Opcode::SUB,
        dest: Some(Operand::Register(RegisterName::EAX)),
        src: Some(Operand::Register(RegisterName::EBX)),
        length: 2,
    };

    execute(&mut cpu, &mut memory, &instruction).unwrap();

    assert_eq!(cpu.registers.get(RegisterName::EAX), 0x0000_5678);
    assert_eq!(cpu.registers.get(RegisterName::EBX), 0x1234_0000);
}

#[test]
fn test_execute_immediate_to_register() {
    let mut cpu = CPU::new();
    let mut memory = Memory::new(1024);

    cpu.registers.set(RegisterName::ECX, 100);

    let instruction = Instruction {
        opcode: Opcode::SUB,
        dest: Some(Operand::Register(RegisterName::ECX)),
        src: Some(Operand::Immediate(50)),
        length: 5,
    };

    execute(&mut cpu, &mut memory, &instruction).unwrap();

    assert_eq!(cpu.registers.get(RegisterName::ECX), 50);
}

#[test]
fn test_execute_with_flags() {
    let mut cpu = CPU::new();
    let mut memory = Memory::new(1024);

    cpu.registers.set(RegisterName::EDX, 0x8000_0000);

    let instruction = Instruction {
        opcode: Opcode::SUB,
        dest: Some(Operand::Register(RegisterName::EDX)),
        src: Some(Operand::Immediate(1)),
        length: 6,
    };

    execute(&mut cpu, &mut memory, &instruction).unwrap();

    assert_eq!(cpu.registers.get(RegisterName::EDX), 0x7FFF_FFFF);
}

#[test]
fn test_execute_invalid_operands() {
    let mut cpu = CPU::new();
    let mut memory = Memory::new(1024);

    let invalid_instruction = Instruction {
        opcode: Opcode::SUB,
        dest: None,
        src: Some(Operand::Immediate(1)),
        length: 1,
    };

    let result = execute(&mut cpu, &mut memory, &invalid_instruction);
    assert!(result.is_err());

    let invalid_instruction = Instruction {
        opcode: Opcode::SUB,
        dest: Some(Operand::Register(RegisterName::EAX)),
        src: None,
        length: 1,
    };

    let result = execute(&mut cpu, &mut memory, &invalid_instruction);
    assert!(result.is_err());
}
