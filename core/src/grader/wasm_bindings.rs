use wasm_bindgen::prelude::*;
use super::api;

#[wasm_bindgen]
pub fn grade_lab(lab_id: u32, program: Vec<u8>) -> String {
    match lab_id {
        1 => api::grade_lab1(&program).to_json(),
        2 => api::grade_lab2(&program).to_json(),
        3 => api::grade_lab3(&program).to_json(),
        _ => {
            // Unknown lab — return empty result
            let result = api::GradingResult {
                earned: 0,
                total: 0,
                auto_max: 0,
                details: vec!["Grading criteria for this lab have not been configured yet.".to_string()],
            };
            result.to_json()
        }
    }
}
