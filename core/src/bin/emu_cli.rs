use std::io::{self, Write};
use web_x86_core::cpu::{RegisterName, Registers};

//map string to registername
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

//parse source operand
fn parse_operand(raw: &str, regs: &Registers) -> Result<u32, String> {
    if let Some(reg_name) = parse_register_name(raw) {
        Ok(regs.get(reg_name))
    } else {
        parse_immediate(raw)
    }
}

//parse numeric intermediate
fn parse_immediate(raw: &str) -> Result<u32, String> {
    let s = raw.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid hex immediate `{s}`: {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid decimal immediate `{s}`: {e}"))
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut regs = Registers::new();

    println!("Simple x86-32 CLI over Registers");
    println!("For now supports:");
    println!("  mov <reg>, <reg|imm>");
    println!("  add <reg>, <reg|imm>");
    println!("Commands: regs, reset, quit\n");

    loop {
        // prompt
        print!("emu> ");
        io::stdout().flush()?;

        // read line
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            // EOF
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // meta-commands
        match line {
            "quit" | "exit" => break,
            "regs" => {
                // Uses your fmt::Display for Registers
                println!("{regs}\n");
                continue;
            }
            "reset" => {
                regs.reset();
                println!("Registers reset.\n");
                continue;
            }
            _ => {}
        }

        //  PARSING
        // Expect: <opcode> <dest>, <src>
        // Example: "mov eax, 5" or "add eax, ebx"
        let mut parts = line.split_whitespace();

        let opcode = match parts.next() {
            Some(op) => op.to_ascii_lowercase(),
            None => continue,
        };

        let dest_raw = match parts.next() {
            Some(d) => d.trim_end_matches(','), // handle "eax,"
            None => {
                eprintln!("error: expected destination register");
                continue;
            }
        };

        let src_raw = match parts.next() {
            Some(s) => s.trim_end_matches(','), // handle "5,"
            None => {
                eprintln!("error: expected source operand (register or value)");
                continue;
            }
        };

        // ensure no extra tokens
        if parts.next().is_some() {
            eprintln!("error: too many tokens; expected: <opcode> <dest>, <src>");
            continue;
        }

        // dest must be a register
        let dest_reg_name = match parse_register_name(dest_raw) {
            Some(r) => r,
            None => {
                eprintln!("error: destination must be a register, got `{dest_raw}`");
                continue;
            }
        };

        // src can be reg or immediate
        let src_val = match parse_operand(src_raw, &regs) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: {e}");
                continue;
            }
        };

        //  EXECUTION (manual for now)
        match opcode.as_str() {
            "mov" => {
                regs.set(dest_reg_name, src_val);
            }
            "add" => {
                let current = regs.get(dest_reg_name);
                regs.set(dest_reg_name, current.wrapping_add(src_val));
            }
            other => {
                eprintln!("error: unsupported opcode `{other}` (only mov/add for now)");
                continue;
            }
        }

        println!("OK. Current registers:");
        println!("{regs}\n");
    }

    Ok(())
}
