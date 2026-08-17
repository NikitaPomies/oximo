//! Tests for Gurobi translation metadata and range semantics.

use oximo_core::prelude::*;
use oximo_gurobi::{Gurobi, GurobiOptions};
use oximo_solver::Solver;

#[test]
fn inverted_linear_range_is_reported_as_infeasible() {
    let m = Model::new("inverted_range");
    variable!(m, x);
    constraint!(m, impossible, 2.0 <= x <= 1.0);
    objective!(m, Min, x);

    let result = Gurobi.solve(&m, &GurobiOptions::default()).expect("solve inverted range");
    assert!(result.termination.is_infeasible(), "termination = {:?}", result.termination);
}
