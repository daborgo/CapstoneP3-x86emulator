// sub.rs
// x86 SUB: dest = dest - src; updates CF, OF, ZF, SF, PF, AF.

#[derive(Default, Debug, Clone, Copy)]
pub struct CpuFlags {
    pub CF: bool, // Carry (borrow for SUB)
    pub PF: bool, // Parity (even parity of low byte)
    pub AF: bool, // Adjust (borrow/carry out of bit 3)
    pub ZF: bool, // Zero
    pub SF: bool, // Sign
    pub OF: bool, // Overflow
}

impl CpuFlags {
    #[inline]
    fn set_szp_u32(&mut self, res: u32, width_bits: u32) {
        let mask = if width_bits == 8 { 0xFF } else if width_bits == 16 { 0xFFFF } else { 0xFFFF_FFFF };
        let v = res & mask;
        self.ZF = v == 0;
        self.SF = ((v >> (width_bits - 1)) & 1) != 0;
        self.PF = even_parity8(v as u8);
    }
}

/// Even parity (true if the number of set bits in the low 8 bits is even)
#[inline]
fn even_parity8(x: u8) -> bool {
    (x.count_ones() & 1) == 0
}

/// Compute OF for subtraction: dest - src (both masked to width)
/// OF = ((dest ^ src) & (dest ^ res) & sign_bit) != 0
#[inline]
fn overflow_sub(dest: u32, src: u32, res: u32, sign_bit: u32) -> bool {
    (((dest ^ src) & (dest ^ res)) & sign_bit) != 0
}

/// Adjust flag (AF) for add/sub can be computed with XOR trick on bit 4.
/// For SUB it’s the same XOR relation:
/// AF = ((dest ^ src ^ res) & 0x10) != 0
#[inline]
fn adjust_flag(dest: u32, src: u32, res: u32) -> bool {
    ((dest ^ src ^ res) & 0x10) != 0
}

/// Core SUB for arbitrary width (8/16/32). Returns (result, updated flags).
#[inline]
fn sub_core(mut flags: CpuFlags, dest: u32, src: u32, width_bits: u32) -> (u32, CpuFlags) {
    let (mask, sign_bit) = match width_bits {
        8  => (0xFFu32, 0x80u32),
        16 => (0xFFFFu32, 0x8000u32),
        32 => (0xFFFF_FFFFu32, 0x8000_0000u32),
        _  => unreachable!("width_bits must be 8/16/32"),
    };

    let d = dest & mask;
    let s = src  & mask;
    let res = d.wrapping_sub(s) & mask;

    // CF: borrow occurred on unsigned subtraction
    flags.CF = d < s;

    // OF: signed overflow
    flags.OF = overflow_sub(d, s, res, sign_bit);

    // AF: borrow/carry out of bit 3 (low nibble)
    flags.AF = adjust_flag(d, s, res);

    // ZF, SF, PF
    flags.set_szp_u32(res, width_bits);

    (res, flags)
}

// Public entry points

#[inline]
pub fn sub8(flags: CpuFlags, dest: u8, src: u8) -> (u8, CpuFlags) {
    let (r, f) = sub_core(flags, dest as u32, src as u32, 8);
    (r as u8, f)
}

#[inline]
pub fn sub16(flags: CpuFlags, dest: u16, src: u16) -> (u16, CpuFlags) {
    let (r, f) = sub_core(flags, dest as u32, src as u32, 16);
    (r as u16, f)
}

#[inline]
pub fn sub32(flags: CpuFlags, dest: u32, src: u32) -> (u32, CpuFlags) {
    sub_core(flags, dest, src, 32)
}

// --------- Tests (you can run with `cargo test`) ----------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub8_basic() {
        // 5 - 3 = 2, no borrow, no overflow
        let (r, f) = sub8(CpuFlags::default(), 5, 3);
        assert_eq!(r, 2);
        assert!(!f.CF);
        assert!(!f.OF);
        assert!(!f.ZF);
        assert!(!f.SF);
        assert_eq!(f.PF, even_parity8(2)); // 0b10 => 1 bit set => odd => PF=false
        assert_eq!(f.AF, adjust_flag(5, 3, 2));
    }

    #[test]
    fn sub8_borrow_cf() {
        // 0x00 - 0x01 = 0xFF, borrow => CF=1, SF=1, ZF=0
        let (r, f) = sub8(CpuFlags::default(), 0x00, 0x01);
        assert_eq!(r, 0xFF);
        assert!(f.CF);
        assert!(!f.ZF);
        assert!(f.SF);
        assert_eq!(f.OF, overflow_sub(0x00, 0x01, 0xFF, 0x80));
    }

    #[test]
    fn sub8_overflow() {
        // (-128) - (1) = 127 in i8 => OF=1 (0x80 - 0x01 = 0x7F)
        let (r, f) = sub8(CpuFlags::default(), 0x80, 0x01);
        assert_eq!(r, 0x7F);
        assert!(f.OF);
        assert!(!f.CF); // unsigned: 0x80 >= 0x01 => no borrow
    }

    #[test]
    fn sub16_zero() {
        // 0x1234 - 0x1234 = 0 -> ZF=1
        let (r, f) = sub16(CpuFlags::default(), 0x1234, 0x1234);
        assert_eq!(r, 0);
        assert!(f.ZF);
        assert!(!f.CF);
        assert!(!f.OF);
        assert!(!f.SF);
    }

    #[test]
    fn sub32_sign() {
        // 0x0000_0001 - 0x0000_0002 = 0xFFFF_FFFF => SF=1, CF=1
        let (r, f) = sub32(CpuFlags::default(), 1, 2);
        assert_eq!(r, 0xFFFF_FFFF);
        assert!(f.SF);
        assert!(f.CF);
        assert!(!f.ZF);
    }
}
