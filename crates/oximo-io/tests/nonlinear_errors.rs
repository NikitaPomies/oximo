//! The LP/MPS writers name where a nonlinear term appears and render the
//! offending sub-expression, instead of a bare "nonlinear" error.

use oximo_core::prelude::*;
use oximo_io::{IoError, to_lp_string, to_mps_string};

fn bilinear_model() -> Model {
    let m = Model::new("bilinear");
    variable!(m, x);
    variable!(m, y);
    objective!(m, Min, x + y);
    constraint!(m, capacity, x * y <= 1.0);
    m
}

fn nonlinear_objective_model() -> Model {
    let m = Model::new("nlobj");
    variable!(m, theta);
    objective!(m, Min, theta.sin());
    constraint!(m, c0, theta >= 0.0);
    m
}

fn nonlinear_constraint_model() -> Model {
    let m = Model::new("nlcon");
    variable!(m, theta);
    objective!(m, Min, theta);
    constraint!(m, capacity, theta.sin() <= 1.0);
    m
}

#[test]
fn lp_names_the_constraint_and_renders_the_term() {
    match to_lp_string(&nonlinear_constraint_model()) {
        Err(IoError::Nonlinear { location, term }) => {
            assert_eq!(location, "constraint \"capacity\"");
            assert_eq!(term, "sin(theta)");
        }
        other => panic!("expected a Nonlinear error, got {other:?}"),
    }
}

#[test]
fn lp_nonlinear_message_is_user_facing() {
    let msg = to_lp_string(&nonlinear_constraint_model()).unwrap_err().to_string();
    assert_eq!(
        msg,
        "expected a linear or quadratic expression in constraint \"capacity\", found nonlinear term: sin(theta)"
    );
}

#[test]
fn lp_names_the_objective() {
    match to_lp_string(&nonlinear_objective_model()) {
        Err(IoError::Nonlinear { location, term }) => {
            assert_eq!(location, "the objective");
            assert_eq!(term, "sin(theta)");
        }
        other => panic!("expected a Nonlinear error, got {other:?}"),
    }
}

#[test]
fn mps_writes_quadratic_constraint() {
    let mps = to_mps_string(&bilinear_model()).expect("quadratic MPS should export");
    assert!(mps.contains("QCMATRIX capacity"), "{mps}");
    assert!(mps.contains("x          y          0.5"), "{mps}");
}

#[test]
fn lp_writes_quadratic_objective_in_cplex_brackets() {
    let m = Model::new("qp");
    variable!(m, x);
    variable!(m, y);
    objective!(m, Min, x.powi(2) + 2.0 * x * y + 3.0 * y);
    let lp = to_lp_string(&m).expect("quadratic objective should export");
    assert!(lp.contains("[ 2 x^2 + 4 x * y ]/2"), "{lp}");
}

#[test]
fn lp_writes_quadratic_constraint_without_objective_scaling() {
    let m = Model::new("qcp");
    variable!(m, x);
    variable!(m, y);
    constraint!(m, q, x.powi(2) + 2.0 * x * y <= 5.0);
    objective!(m, Min, x + y);
    let lp = to_lp_string(&m).expect("quadratic constraint should export");
    assert!(lp.contains("[ x^2 + 2 x * y ] <= 5"), "{lp}");
}

#[test]
fn lp_writes_negative_objective_constant_with_a_sign() {
    let m = Model::new("constant");
    variable!(m, x);
    objective!(m, Min, x - 4.0);
    let lp = to_lp_string(&m).expect("objective constant should export");
    assert!(lp.contains("obj: x - 4"), "{lp}");
}
