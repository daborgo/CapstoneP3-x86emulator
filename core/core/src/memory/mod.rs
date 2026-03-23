//! Memory Module
//! 
//! This module contains the memory system including RAM and MMIO devices.

pub mod ram;

// Re-export main types for easier access
pub use ram::{Memory, MemoryError, IoDevices};

