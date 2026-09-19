//! A feasibility model (`objective!(m, Feasibility)`) exports to every text
//! format: LP and MPS write a zero objective, NL declares zero objectives.

use oximo_core::prelude::*;
use oximo_io::{read_nl, to_lp_string, to_mps_string, to_nl_string};

fn feasibility_model() -> Model {
    let m = Model::new("feas");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, c0, x + y >= 1.0);
    constraint!(m, c1, x - y <= 2.0);
    objective!(m, Feasibility);
    m
}

#[test]
fn lp_writes_zero_objective() {
    let text = to_lp_string(&feasibility_model()).expect("write lp");
    assert!(text.contains("Minimize\n obj: 0\n"), "zero objective row: {text}");
    assert!(text.contains("c0: x + y >= 1"), "constraints still written: {text}");
}

#[test]
fn mps_writes_empty_objective_row() {
    let text = to_mps_string(&feasibility_model()).expect("write mps");
    assert!(text.contains("OBJSENSE\n MIN\n"), "minimize by convention: {text}");
    assert!(text.contains(" N  OBJ\n"), "free objective row declared: {text}");
    let columns = text
        .split_once("COLUMNS\n")
        .and_then(|(_, rest)| rest.split_once("RHS\n"))
        .map(|(columns, _)| columns)
        .expect("COLUMNS section");
    assert!(
        !columns.contains("OBJ"),
        "zero objective must not have any OBJ coefficients: {columns}"
    );
}

#[test]
fn nl_writes_zero_objectives() {
    let text = to_nl_string(&feasibility_model()).expect("write nl");
    let counts = text.lines().nth(1).expect("header line 2");
    assert!(counts.starts_with(" 2 2 0 0 0"), "zero objectives declared: {counts}");
    assert!(!text.contains("\nO0"), "no O segment: {text}");
    assert!(!text.contains("\nG0"), "no gradient segment: {text}");
    // NL is the one format that represents feasibility natively, so it round-trips.
    let reread = read_nl(text.as_bytes()).expect("read nl");
    assert!(reread.is_feasibility());
    assert_eq!(reread.constraints().algebraic().len(), 2);
}
