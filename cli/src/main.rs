use std::fmt;

use emu_core::cpu::registers::RegisterName;
use emu_core::cpu::CPU;
use emu_core::decoder::{Instruction, Opcode, Operand};
use emu_core::instructions;
use emu_core::instructions::InstructionError;
use emu_core::memory::Memory;

use std::io::{self, Write};

fn main() {
    let mut cpu = Cpu::new();
    let mut memory = Memory::new();

    println!("Welcome to the x86 emulator");
    println!("Type an instruction (ex: add rax 5)");
    println!("Commands: regs | quit");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().expect("failed to flush");

        //read input
        let mut line = String::new();
        let bytes = io::stdin().read_line(&mut line).expect("failed to read");

        if bytes == 0 {
            break;
        }

        let line = line.trim();

        //empty input
        if line.is_empty() {
            continue;
        }

        //quit command
        if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
            println!("Exiting the CLI");
            break;
        }

        //show regs
        if line.eq_ignore_ascii_case("regs") {
            dump_registers(&cpu);
            continue;
        }

        //expect format operand reg val
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            println!("expected <op> <reg> <val>");
            continue;
        }

        let op = parts[0].to_lowercase();
        let reg_str = parts[1].to_lowercase();
        let imm_str = parts[2].to_lowercase();

        //parse reg
        let reg = match parse_register(&reg_str) {
            Some(r) => r,
            None => {
                println!("Unknown register: {}", reg_str);
                continue;
            }
        };

        //parse constant
        let imm = match imm_str.parse() {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid value: {}", const_str);
                continue;
            }
        };

        //only add/mov so far
        if op != "add" && op != "mov" {
            println!("Unknown opcode: {}", op);
            println!("Add, mov");
            continue;
        }

        //build instruction
        let instr = match build_reg_imm_instr(&op, reg, imm) {
            Some(i) => i,
            None => {
                println!("Unsupported {op} (reg, imm) in CLI");
                continue;
            }
        };

        //call dispatcher
        match instructions::execute(&mut cpu, &mut memory, &instr) {
            Ok(()) => {
                println!("Executed: {} {:?}, {}", op, reg, imm);
                dump_registers(&cpu);
            }
            Err(e) => {
                println!("Error executing {}: {}", op, e);
            }
        }
    }
}

fn dump_registers(cpu: &Cpu) {
    println!("{}", cpu.registers);
}

fn parse_register(name: &str) -> Option<Register> {
    match name.to_ascii_lowercase().as_str() {
        "eax" => Some(RegisterName::EAX),
        "ebx" => Some(RegisterName::EBX),
        "ecx" => Some(RegisterName::ECX),
        "edx" => Some(RegisterName::EDX),
        "esi" => Some(RegisterName::ESI),
        "edi" => Some(RegisterName::EDI),
        "ebp" => Some(RegisterName::EBP),
        "esp" => Some(RegisterName::ESP),
        "eip" => Some(RegisterName::EIP),
        _ => None,
    }
}

fn build_reg_imm_instr(code: &str, reg: RegisterName, imm: u32) -> Option<Instruction> {
    let opcode = match code {
        "add" => Opcode::ADD,
        "mov" => Opcode::MOV,
        _ => return None,
    };

    Some(Instruction {
        opcode,
        dest: Some(Operand::Register(reg)),
        src: Some(Operand::Immediate(imm)),
        length: 5,
        ..Default::default()
    })
}
