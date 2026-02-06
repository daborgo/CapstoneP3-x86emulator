use std::io::{self, Write};

use web_x86_core::cpu::Registers;
use web_x86_core::cpu::{RegisterName, CPU};
use web_x86_core::decoder::Operand;
use web_x86_core::decoder::{Instruction, Opcode};
use web_x86_core::instructions::{add, call, jmp, mov, pop, push, ret, sub};
use web_x86_core::memory::Memory;

fn parse_register_name(s: &str) -> Option<RegisterName> {
    match s.to_ascii_lowercase().as_str() {
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

fn parse_u32(raw: &str) -> Result<u32, String> {
    let s = raw.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid hex `{s}`: {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid decimal `{s}`: {e}"))
    }
}

/// CLI supports:
/// - Register: eax
/// - Immediate: 5 or 0x10
/// - Memory: [0x1000]
fn parse_operand(raw: &str) -> Result<Operand, String> {
    let s = raw.trim();

    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len() - 1].trim();
        let addr = parse_u32(inner)?;
        return Ok(Operand::Memory(addr));
    }

    if let Some(r) = parse_register_name(s) {
        Ok(Operand::Register(r))
    } else {
        Ok(Operand::Immediate(parse_u32(s)?))
    }
}

#[derive(Debug, Clone)]
enum CliInst {
    // 2-operand
    Mov { dst: Operand, src: Operand },
    // Add { dst: Operand, src: Operand },
    Sub { dst: Operand, src: Operand },
    // Cmp { dst: Operand, src: Operand },

    // 1-operand
    Push { src: Operand },
    Pop { dst: Operand },
    // Jmp { target: Operand },
    Call { target: Operand },

    // 0-operand
    Ret,
}

fn parse_cli_inst(line: &str) -> Result<CliInst, String> {
    let mut parts = line.split_whitespace();

    let opcode = parts.next().ok_or("expected opcode")?.to_ascii_lowercase();

    let a = parts.next().map(|s| s.trim_end_matches(','));
    let b = parts.next().map(|s| s.trim_end_matches(','));

    if parts.next().is_some() {
        return Err("too many tokens".into());
    }

    match opcode.as_str() {
        // 0-operand
        "ret" => {
            if a.is_some() || b.is_some() {
                return Err("usage: ret".into());
            }
            Ok(CliInst::Ret)
        }

        // 1-operand
        "push" => {
            let src_raw = a.ok_or("usage: push <reg|imm|[addr]>")?;
            if b.is_some() {
                return Err("push takes one operand".into());
            }
            Ok(CliInst::Push {
                src: parse_operand(src_raw)?,
            })
        }
        "pop" => {
            let dst_raw = a.ok_or("usage: pop <reg|[addr]>")?;
            if b.is_some() {
                return Err("pop takes one operand".into());
            }
            Ok(CliInst::Pop {
                dst: parse_operand(dst_raw)?,
            })
        }
        // "jmp" => {
        //     let t = a.ok_or("usage: jmp <reg|imm|[addr]>")?;
        //     if b.is_some() {
        //         return Err("jmp takes one operand".into());
        //     }
        //     Ok(CliInst::Jmp {
        //         target: parse_operand(t)?,
        //     })
        // }
        "call" => {
            let t = a.ok_or("usage: call <reg|imm|[addr]>")?;
            if b.is_some() {
                return Err("call takes one operand".into());
            }
            Ok(CliInst::Call {
                target: parse_operand(t)?,
            })
        }

        // 2-operand
        "mov" | "sub" => {
            let left_raw = a.ok_or_else(|| format!("usage: {opcode} <dst>, <src>"))?;
            let right_raw = b.ok_or_else(|| format!("usage: {opcode} <dst>, <src>"))?;

            let dst = parse_operand(left_raw)?;
            let src = parse_operand(right_raw)?;

            Ok(match opcode.as_str() {
                "mov" => CliInst::Mov { dst, src },
                //"add" => CliInst::Add { dst, src },
                "sub" => CliInst::Sub { dst, src },
                // "cmp" => CliInst::Cmp { dst, src },
                _ => unreachable!(),
            })
        }

        other => Err(format!("unsupported opcode `{other}`")),
    }
}

fn make_instruction(opcode: Opcode, dest: Option<Operand>, src: Option<Operand>) -> Instruction {
    Instruction {
        opcode,
        dest,
        src,
        length: 1,
    }
}

fn _eval_operand(op: &Operand, regs: &Registers, memory: &Memory) -> Result<u32, String> {
    match op {
        Operand::Register(r) => Ok(regs.get(*r)),
        Operand::Immediate(v) => Ok(*v),
        Operand::Memory(addr) => memory.read_u32(*addr).map_err(|e| format!("{e:?}")),
    }
}

fn err_to_string<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

fn execute_via_core(cli: CliInst, cpu: &mut CPU, memory: &mut Memory) -> Result<(), String> {
    // Your instruction fns return Result<(), ExecutionError>.
    // We convert that to String.

    match cli {
        // CliInst::Add { dst, src } => {
        //     let inst = make_instruction(Opcode::, Some(dst), Some(src));
        //     add::add(cpu, memory, &inst).map_err(err_to_string)
        // }
        CliInst::Sub { dst, src } => {
            let inst = make_instruction(Opcode::SUB, Some(dst), Some(src));
            sub::execute(cpu, memory, &inst).map_err(err_to_string)
        }
        CliInst::Mov { dst, src } => {
            let inst = make_instruction(Opcode::MOV, Some(dst), Some(src));
            mov::execute(cpu, memory, &inst).map_err(err_to_string)
        }

        // These operand placements (dest vs src) must match how your instruction files read operands.
        // If any module expects the operand in `dest` instead of `src`, swap it here.
        CliInst::Push { src } => {
            let inst = make_instruction(Opcode::PUSH, None, Some(src));
            push::execute(cpu, memory, &inst).map_err(err_to_string)
        }
        CliInst::Pop { dst } => {
            let inst = make_instruction(Opcode::POP, Some(dst), None);
            pop::execute(cpu, memory, &inst).map_err(err_to_string)
        }

        // CliInst::Jmp { target } => {
        //     let inst = make_instruction(Opcode::JMP, None, Some(target));
        //     jmp::execute(cpu, memory, &inst).map_err(err_to_string)
        // }
        CliInst::Call { target } => {
            let inst = make_instruction(Opcode::CALL, None, Some(target));
            call::execute(cpu, memory, &inst).map_err(err_to_string)
        }
        CliInst::Ret => {
            let inst = make_instruction(Opcode::RET, None, None);
            ret::execute(cpu, memory, &inst).map_err(err_to_string)
        }
    }
}

fn print_help() {
    println!("Simple x86-32 CLI");
    println!("Meta: regs, reset, help, quit/exit\n");
    println!("Operands:");
    println!("  reg: eax");
    println!("  imm: 5 or 0x10");
    println!("  mem: [0x1000]\n");
    println!("Opcodes:");
    println!("  mov <dst>, <src>");
    // println!("  add <dst>, <src>");
    println!("  sub <dst>, <src>");
    //println!("  cmp <dst>, <src>");
    println!("  push <src>");
    println!("  pop <dst>");
    // println!("  jmp <target>");
    println!("  call <target>");
    println!("  ret\n");
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();

    let mut cpu = CPU::new();
    let mut memory = Memory::new(16 * 1024 * 1024);

    print_help();

    loop {
        print!("emu> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "quit" | "exit" => break,
            "help" => {
                print_help();
                continue;
            }
            "regs" => {
                println!("{}", cpu.registers);
                println!(
                    "EIP: 0x{:08X}  ESP: 0x{:08X}\n",
                    cpu.registers.eip, cpu.registers.esp
                );
                continue;
            }
            "reset" => {
                cpu.reset();
                println!("CPU reset.\n");
                continue;
            }
            _ => {}
        }

        match parse_cli_inst(line) {
            Ok(cli_inst) => match execute_via_core(cli_inst, &mut cpu, &mut memory) {
                Ok(()) => {
                    println!("OK");
                    println!("{}", cpu.registers);
                    println!(
                        "EIP: 0x{:08X}  ESP: 0x{:08X}\n",
                        cpu.registers.eip, cpu.registers.esp
                    );
                }
                Err(e) => {
                    eprintln!("error: {e}\n");
                }
            },
            Err(e) => {
                eprintln!("error: {e}\n");
            }
        }
    }

    Ok(())
}
