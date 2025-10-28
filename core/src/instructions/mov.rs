use crate::cpu::{CPU, Operand};

pub fn mov(cpu: &mut CPU, dest: Operand, src: Operand) {
    let value = match src {
        Operand::Reg(r) => cpu.regs[r],
        Operand::Imm(imm) => imm,
        Operand::Mem(addr) => {
            // Load little-endian 32-bit value from memory
            let bytes = &cpu.memory[addr as usize..addr as usize + 4];
            u32::from_le_bytes(bytes.try_into().unwrap())
        }
    };

    match dest {
        Operand::Reg(r) => cpu.regs[r] = value,
        Operand::Mem(addr) => {
            let bytes = value.to_le_bytes();
            cpu.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        }
        Operand::Imm(_) => panic!("Cannot move into an immediate value!"),
    }
}
