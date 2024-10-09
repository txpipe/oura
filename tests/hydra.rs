use std::fs;

use oura::sources::hydra::{HydraMessage, HydraMessagePayload};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn run_scenario(_expected_msgs: &[HydraMessage], expected_file: &str) -> TestResult {
    let _file = fs::read_to_string(expected_file)?;
    Ok(())
}

#[test]
fn hydra_scenario_1() -> TestResult {
    let msgs = [ HydraMessage { seq: 0, payload: HydraMessagePayload::Other } ];
    run_scenario(&msgs, "tests/hydra/scenario_1.txt")
}

#[test]
fn hydra_scenario_2() -> TestResult {
    let msgs = [ HydraMessage { seq: 0, payload: HydraMessagePayload::Other } ];
    run_scenario(&msgs, "tests/hydra/scenario_2.txt")
}
