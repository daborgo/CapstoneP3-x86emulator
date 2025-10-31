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

    /// Execute one instruction using fetch-decode-execute cycle
    ///
    /// This implements the complete CPU cycle:
    /// 1. FETCH: Read instruction bytes from memory at EIP
    /// 2. DECODE: Parse bytes into structured instruction
    /// 3. EXECUTE: Execute the instruction
    /// 4. Update step counter
    pub fn step(&mut self) -> u64 {
        self.steps += 1;

        // TODO: Implement actual fetch-decode-execute cycle
        // For now, just increment the counter
        //
        // The real implementation would be:
        // 1. let bytes = self.memory.read_bytes(self.cpu.registers.eip, 15)?;
        // 2. let instruction = decode(&bytes)?;
        // 3. execute(&mut self.cpu, &mut self.memory, &instruction)?;

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

    /// Set EAX register value (for testing)
    pub fn set_eax(&mut self, value: u32) {
        self.cpu.registers.eax = value;
    }

    /// Get EIP (instruction pointer) value
    pub fn get_eip(&self) -> u32 {
        self.cpu.registers.eip
    }

    /// Get ESP (stack pointer) value
    pub fn get_esp(&self) -> u32 {
        self.cpu.registers.esp
    }

    /// Reset the emulator to initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory = Memory::default();
        self.steps = 0;
    }
}
