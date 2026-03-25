use crate::Emulator;

const LOAD_ADDR: u32 = 0x0000_1000;
const ARRAY_BASE: u32 = 0x0000_2000;
const MAX_STEPS: u32 = 5000;
const POINTS_PER_TEST: u32 = 8;
const MANUAL_POINTS: u32 = 6;

struct TestCase {
    a0: u32,
    a1: u32,
    exp_a2: u32,
    exp_a3: u32,
    exp_a4: u32,
    exp_a5: u32,
    label: &'static str,
}

const LAB1_TESTS: &[TestCase] = &[
    TestCase {
        a0: 4,
        a1: 5,
        exp_a2: 20,
        exp_a3: 0,
        exp_a4: 0,
        exp_a5: 80,
        label: "A[0]=4, A[1]=5",
    },
    TestCase {
        a0: 0xFFFF_FFF8, // -8 as u32
        a1: 0xFFFF_FFFC, // -4 as u32
        exp_a2: 32,
        exp_a3: 0xFFFF_FFF4, // -12 as u32
        exp_a4: 0,
        exp_a5: 128,
        label: "A[0]=-8, A[1]=-4",
    },
    TestCase {
        a0: 0xFFFF_FFFB, // -5 as u32
        a1: 8,
        exp_a2: 0xFFFF_FFD8, // -40 as u32
        exp_a3: 7,
        exp_a4: 0,
        exp_a5: 0xFF08_FF60, // -16187552 as u32
        label: "A[0]=-5, A[1]=8",
    },
];

pub struct GradingResult {
    pub earned: u32,
    pub total: u32,
    pub auto_max: u32,
    pub details: Vec<String>,
}

impl GradingResult {
    pub fn to_json(&self) -> String {
        let details_json: Vec<String> = self
            .details
            .iter()
            .map(|d| format!("\"{}\"", d.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect();

        format!(
            "{{\"earned\":{},\"total\":{},\"autoMax\":{},\"details\":[{}]}}",
            self.earned, self.total, self.auto_max, details_json.join(",")
        )
    }
}

fn run_lab1_test(tc: &TestCase, program: &[u8]) -> Result<(bool, Vec<String>), String> {
    let mut emu = Emulator::new();
    emu.load_program(program.to_vec(), LOAD_ADDR)?;
    emu.write_u32(ARRAY_BASE, tc.a0)?;
    emu.write_u32(ARRAY_BASE + 4, tc.a1)?;

    for _ in 0..MAX_STEPS {
        emu.step();
    }

    let a2 = emu.read_u32(ARRAY_BASE + 8)?;
    let a3 = emu.read_u32(ARRAY_BASE + 12)?;
    let a4 = emu.read_u32(ARRAY_BASE + 16)?;
    let a5 = emu.read_u32(ARRAY_BASE + 20)?;

    let ok = a2 == tc.exp_a2 && a3 == tc.exp_a3 && a4 == tc.exp_a4 && a5 == tc.exp_a5;

    let mut mismatches = Vec::new();
    if !ok {
        if a2 != tc.exp_a2 {
            mismatches.push(format!("    A[2]: expected 0x{:X} got 0x{:X}", tc.exp_a2, a2));
        }
        if a3 != tc.exp_a3 {
            mismatches.push(format!("    A[3]: expected 0x{:X} got 0x{:X}", tc.exp_a3, a3));
        }
        if a4 != tc.exp_a4 {
            mismatches.push(format!("    A[4]: expected 0x{:X} got 0x{:X}", tc.exp_a4, a4));
        }
        if a5 != tc.exp_a5 {
            mismatches.push(format!("    A[5]: expected 0x{:X} got 0x{:X}", tc.exp_a5, a5));
        }
    }

    Ok((ok, mismatches))
}

pub fn grade_lab1(program: &[u8]) -> GradingResult {
    let auto_max = LAB1_TESTS.len() as u32 * POINTS_PER_TEST;
    let total = auto_max + MANUAL_POINTS;
    let mut earned: u32 = 0;
    let mut details: Vec<String> = Vec::new();

    for tc in LAB1_TESTS {
        match run_lab1_test(tc, program) {
            Ok((true, _)) => {
                earned += POINTS_PER_TEST;
                details.push(format!(
                    "\u{2713} {} ({}/{} pts)",
                    tc.label, POINTS_PER_TEST, POINTS_PER_TEST
                ));
            }
            Ok((false, mismatches)) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts)",
                    tc.label, POINTS_PER_TEST
                ));
                details.extend(mismatches);
            }
            Err(e) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts) - runtime error: {}",
                    tc.label, POINTS_PER_TEST, e
                ));
            }
        }
    }

    GradingResult {
        earned,
        total,
        auto_max,
        details,
    }
}

// ─── Lab 2: Remove element from array ────────────────────────────────────────

const LAB2_POINTS_PER_TEST: u32 = 5;
const LAB2_MANUAL_POINTS: u32 = 10; // 2 manual checks x 5 pts each
const N_ADDR: u32 = 0x0000_1F00;     // memory location for n
const VAL_ADDR: u32 = 0x0000_1F04;   // memory location for val

struct Lab2TestCase {
    n: u32,
    val: i32,
    array: &'static [i32],
    exp_n: u32,
    exp_array: &'static [i32],
    label: &'static str,
}

const LAB2_TESTS: &[Lab2TestCase] = &[
    Lab2TestCase {
        n: 10,
        val: -15,
        array: &[4, 8, -15, 5, -10, -15, 0, -15, 40, -15],
        exp_n: 6,
        exp_array: &[4, 8, 5, -10, 0, 40],
        label: "Remove -15 from [4,8,-15,5,-10,-15,0,-15,40,-15]",
    },
    Lab2TestCase {
        n: 4,
        val: 0,
        array: &[-8, -4, 4, 8],
        exp_n: 4,
        exp_array: &[-8, -4, 4, 8],
        label: "Remove 0 from [-8,-4,4,8] (no match)",
    },
    Lab2TestCase {
        n: 8,
        val: 100,
        array: &[-50, 80, 100, 0, -80, -50, -100, 0],
        exp_n: 7,
        exp_array: &[-50, 80, 0, -80, -50, -100, 0],
        label: "Remove 100 from [-50,80,100,0,-80,-50,-100,0]",
    },
    Lab2TestCase {
        n: 5,
        val: 230,
        array: &[230, 230, 230, 230, 230],
        exp_n: 0,
        exp_array: &[],
        label: "Remove 230 from [230,230,230,230,230] (all removed)",
    },
];

fn run_lab2_test(tc: &Lab2TestCase, program: &[u8]) -> Result<(bool, Vec<String>), String> {
    let mut emu = Emulator::new();
    emu.load_program(program.to_vec(), LOAD_ADDR)?;

    // Write n and val to memory
    emu.write_u32(N_ADDR, tc.n)?;
    emu.write_u32(VAL_ADDR, tc.val as u32)?;

    // Write the initial array to memory at ARRAY_BASE
    for (i, &elem) in tc.array.iter().enumerate() {
        emu.write_u32(ARRAY_BASE + (i as u32) * 4, elem as u32)?;
    }

    // Run the program
    for _ in 0..MAX_STEPS {
        emu.step();
    }

    // Read the resulting n
    let result_n = emu.read_u32(N_ADDR)?;

    // Check n first
    let mut mismatches = Vec::new();
    let mut ok = true;

    if result_n != tc.exp_n {
        ok = false;
        mismatches.push(format!("    n: expected {} got {}", tc.exp_n, result_n));
    }

    // Check the first exp_n elements of the array
    for i in 0..tc.exp_n {
        let got = emu.read_u32(ARRAY_BASE + i * 4)?;
        let expected = tc.exp_array[i as usize] as u32;
        if got != expected {
            ok = false;
            mismatches.push(format!(
                "    A[{}]: expected {} got {}",
                i, tc.exp_array[i as usize], got as i32
            ));
        }
    }

    Ok((ok, mismatches))
}

pub fn grade_lab2(program: &[u8]) -> GradingResult {
    let auto_max = LAB2_TESTS.len() as u32 * LAB2_POINTS_PER_TEST;
    let total = auto_max + LAB2_MANUAL_POINTS;
    let mut earned: u32 = 0;
    let mut details: Vec<String> = Vec::new();

    for tc in LAB2_TESTS {
        match run_lab2_test(tc, program) {
            Ok((true, _)) => {
                earned += LAB2_POINTS_PER_TEST;
                details.push(format!(
                    "\u{2713} {} ({}/{} pts)",
                    tc.label, LAB2_POINTS_PER_TEST, LAB2_POINTS_PER_TEST
                ));
            }
            Ok((false, mismatches)) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts)",
                    tc.label, LAB2_POINTS_PER_TEST
                ));
                details.extend(mismatches);
            }
            Err(e) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts) - runtime error: {}",
                    tc.label, LAB2_POINTS_PER_TEST, e
                ));
            }
        }
    }

    GradingResult {
        earned,
        total,
        auto_max,
        details,
    }
}

// ─── Lab 3: Single Procedure Call (exponent array) ───────────────────────────

const LAB3_POINTS_PER_TEST: u32 = 5;
const LAB3_MANUAL_POINTS: u32 = 10; // comments (5) + procedure call usage (5)
const N1_ADDR: u32 = 0x0000_1F00;   // memory location for n1
const N2_ADDR: u32 = 0x0000_1F04;   // memory location for n2 (result)
const A_BASE: u32 = 0x0000_2000;    // array A base
const B_BASE: u32 = 0x0000_3000;    // array B base

struct Lab3TestCase {
    n1: u32,
    array_a: &'static [i32],
    exp_n2: u32,
    exp_b: &'static [i32],
    label: &'static str,
}

/// Compute x^y as i32 (wrapping multiplication, matching 32-bit behavior)
#[allow(dead_code)]
fn ipow(x: i32, y: u32) -> i32 {
    let mut result: i32 = 1;
    for _ in 0..y {
        result = result.wrapping_mul(x);
    }
    result
}

const LAB3_TESTS: &[Lab3TestCase] = &[
    Lab3TestCase {
        n1: 5,
        array_a: &[10, 5, -5, -2, 0],
        exp_n2: 5,
        exp_b: &[1, 5, 25, -8, 0],
        label: "A=[10,5,-5,-2,0] n1=5",
    },
    Lab3TestCase {
        n1: 1,
        array_a: &[-8],
        exp_n2: 1,
        exp_b: &[1],
        label: "A=[-8] n1=1 (single element)",
    },
    Lab3TestCase {
        n1: 10,
        array_a: &[-5, 8, 4, -6, 6, 0, -3, -2, 2, -2],
        exp_n2: 10,
        exp_b: &[1, 8, 16, -216, 1296, 0, 729, -128, 256, -512],
        label: "A=[-5,8,4,-6,6,0,-3,-2,2,-2] n1=10",
    },
    Lab3TestCase {
        n1: 8,
        array_a: &[-80, -40, 25, 20, -15, 10, -8, -4],
        exp_n2: 8,
        exp_b: &[1, -40, 625, 8000, 50625, 100000, 262144, -16384],
        label: "A=[-80,-40,25,20,-15,10,-8,-4] n1=8",
    },
];

fn run_lab3_test(tc: &Lab3TestCase, program: &[u8]) -> Result<(bool, Vec<String>), String> {
    let mut emu = Emulator::new();
    emu.load_program(program.to_vec(), LOAD_ADDR)?;

    // Write n1
    emu.write_u32(N1_ADDR, tc.n1)?;
    // Clear n2
    emu.write_u32(N2_ADDR, 0)?;

    // Write array A
    for (i, &elem) in tc.array_a.iter().enumerate() {
        emu.write_u32(A_BASE + (i as u32) * 4, elem as u32)?;
    }

    // Run the program
    for _ in 0..50000 {
        emu.step();
    }

    // Check n2
    let result_n2 = emu.read_u32(N2_ADDR)?;
    let mut mismatches = Vec::new();
    let mut ok = true;

    if result_n2 != tc.exp_n2 {
        ok = false;
        mismatches.push(format!("    n2: expected {} got {}", tc.exp_n2, result_n2));
    }

    // Check B array
    for i in 0..tc.exp_n2 {
        let got = emu.read_u32(B_BASE + i * 4)?;
        let expected = tc.exp_b[i as usize] as u32;
        if got != expected {
            ok = false;
            mismatches.push(format!(
                "    B[{}]: expected {} got {}",
                i, tc.exp_b[i as usize], got as i32
            ));
        }
    }

    Ok((ok, mismatches))
}

pub fn grade_lab3(program: &[u8]) -> GradingResult {
    let auto_max = LAB3_TESTS.len() as u32 * LAB3_POINTS_PER_TEST;
    let total = auto_max + LAB3_MANUAL_POINTS;
    let mut earned: u32 = 0;
    let mut details: Vec<String> = Vec::new();

    for tc in LAB3_TESTS {
        match run_lab3_test(tc, program) {
            Ok((true, _)) => {
                earned += LAB3_POINTS_PER_TEST;
                details.push(format!(
                    "\u{2713} {} ({}/{} pts)",
                    tc.label, LAB3_POINTS_PER_TEST, LAB3_POINTS_PER_TEST
                ));
            }
            Ok((false, mismatches)) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts)",
                    tc.label, LAB3_POINTS_PER_TEST
                ));
                details.extend(mismatches);
            }
            Err(e) => {
                details.push(format!(
                    "\u{2717} {} (0/{} pts) - runtime error: {}",
                    tc.label, LAB3_POINTS_PER_TEST, e
                ));
            }
        }
    }

    GradingResult {
        earned,
        total,
        auto_max,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grading_result_to_json() {
        let result = GradingResult {
            earned: 24,
            total: 30,
            auto_max: 24,
            details: vec!["test passed".to_string()],
        };
        let json = result.to_json();
        assert!(json.contains("\"earned\":24"));
        assert!(json.contains("\"total\":30"));
        assert!(json.contains("\"autoMax\":24"));
        assert!(json.contains("\"test passed\""));
    }
}
