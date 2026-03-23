//! Web x86 Emulator Core
//!
//! This is the main entry point for the WASM-compiled x86 emulator.
//! It provides the public API that the frontend can call.

use wasm_bindgen::prelude::*;

// Declare modules
pub mod cpu;
pub mod decoder;
pub mod instructions;
pub mod memory;

// Use the types we need
use cpu::CPU;
use decoder::decode;
use instructions::execute;
use memory::Memory;

/// Main Emulator structure exposed to JavaScript
///
/// This struct combines the CPU and Memory and provides
/// the public API for the frontend.
#[wasm_bindgen]
pub struct Emulator {
    /// CPU state (registers, flags)
    cpu: CPU,

    /// Memory system (RAM, MMIO)
    memory: Memory,

    /// Step counter for debugging
    steps: u64,
}

#[wasm_bindgen]
impl Emulator {
    /// Create a new emulator instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            memory: Memory::default(),
            steps: 0,
        }
    }

    /// Load a program's raw bytes into memory at a given address and set EIP there
    /// Returns Err(String) on memory write failure.
    pub fn load_program(&mut self, program: Vec<u8>, load_address: u32) -> Result<(), String> {
        for (i, byte) in program.iter().enumerate() {
            let addr = load_address.wrapping_add(i as u32);
            self.memory
                .write_u8(addr, *byte)
                .map_err(|e| e.to_string())?;
        }
        self.cpu.registers.eip = load_address;
        Ok(())
    }

    /// Execute one instruction using fetch-decode-execute cycle
    ///
    /// This implements the complete CPU cycle:
    /// 1. FETCH: Read instruction bytes from memory at EIP
    /// 2. DECODE: Parse bytes into structured instruction
    /// 3. EXECUTE: Execute the instruction
    /// 4. Update step counter
    pub fn step(&mut self) -> u64 {
        // Fetch up to 15 bytes from memory starting at EIP
        let eip = self.cpu.registers.eip;
        let mut buffer = [0u8; 15];
        let mut fetched = 0usize;
        for i in 0..buffer.len() {
            match self.memory.read_u8(eip.wrapping_add(i as u32)) {
                Ok(b) => {
                    buffer[i] = b;
                    fetched += 1;
                }
                Err(_) => break,
            }
        }

        if fetched == 0 {
            // Nothing to do if we can't read memory at EIP
            return self.steps;
        }

        // Always increment step counter (even if decode/execute fails)
        self.steps += 1;

        match decode(&buffer[..fetched]) {
            Ok(instruction) => {
                // Execute the decoded instruction
                let _ = execute(&mut self.cpu, &mut self.memory, &instruction);
            }
            Err(_e) => {
                // If we cannot decode, advance by 1 byte to avoid infinite loop
                self.cpu.registers.advance_ip(1);
            }
        }

        self.steps
    }

    /// Get the number of steps executed
    pub fn get_steps(&self) -> u64 {
        self.steps
    }

    /// Get EAX register value (for testing)
    pub fn get_eax(&self) -> u32 {
        self.cpu.registers.eax
    }

    /// Additional register getters useful for UI
    pub fn get_ebx(&self) -> u32 { self.cpu.registers.ebx }
    pub fn get_ecx(&self) -> u32 { self.cpu.registers.ecx }
    pub fn get_edx(&self) -> u32 { self.cpu.registers.edx }
    pub fn get_ebp(&self) -> u32 { self.cpu.registers.ebp }
    pub fn get_esi(&self) -> u32 { self.cpu.registers.esi }
    pub fn get_edi(&self) -> u32 { self.cpu.registers.edi }
    
    /// Set EAX register value (for testing)
    pub fn set_eax(&mut self, value: u32) {
        self.cpu.registers.eax = value;
    }
    
    /// Set EBX register value
    pub fn set_ebx(&mut self, value: u32) {
        self.cpu.registers.ebx = value;
    }
    
    /// Set ECX register value
    pub fn set_ecx(&mut self, value: u32) {
        self.cpu.registers.ecx = value;
    }
    
    /// Set EDX register value
    pub fn set_edx(&mut self, value: u32) {
        self.cpu.registers.edx = value;
    }
    
    /// Set EBP register value
    pub fn set_ebp(&mut self, value: u32) {
        self.cpu.registers.ebp = value;
    }
    
    /// Set ESP register value
    pub fn set_esp(&mut self, value: u32) {
        self.cpu.registers.esp = value;
    }
    
    /// Set ESI register value
    pub fn set_esi(&mut self, value: u32) {
        self.cpu.registers.esi = value;
    }
    
    /// Set EDI register value
    pub fn set_edi(&mut self, value: u32) {
        self.cpu.registers.edi = value;
    }
    
    /// Set EIP (instruction pointer) value
    pub fn set_eip(&mut self, value: u32) {
        self.cpu.registers.eip = value;
    }

    /// Get EIP (instruction pointer) value
    pub fn get_eip(&self) -> u32 {
        self.cpu.registers.eip
    }

    /// Get ESP (stack pointer) value
    pub fn get_esp(&self) -> u32 {
        self.cpu.registers.esp
    }

    /// Flag getters
    pub fn get_cf(&self) -> bool { self.cpu.flags.cf }
    pub fn get_pf(&self) -> bool { self.cpu.flags.pf }
    pub fn get_af(&self) -> bool { self.cpu.flags.af }
    pub fn get_zf(&self) -> bool { self.cpu.flags.zf }
    pub fn get_sf(&self) -> bool { self.cpu.flags.sf }
    pub fn get_of(&self) -> bool { self.cpu.flags.of }
    
    /// Reset the emulator to initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory = Memory::default();
        self.steps = 0;
    }
}
