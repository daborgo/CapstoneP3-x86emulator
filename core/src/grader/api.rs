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
