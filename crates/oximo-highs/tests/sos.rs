use oximo_core::prelude::*;
use oximo_highs::{Highs, HighsOptions};
use oximo_solver::{PersistentSolver, Solver, SolverError};

fn sos_model() -> Model {
    let m = Model::new("highs_sos");
    variable!(m, x);
    variable!(m, y);
    sos_constraint!(m, choice, SOS1, [x, y]);
    m
}

#[test]
fn highs_rejects_sos_constraints() {
    let model = sos_model();
    let mut solver = Highs;
    assert!(matches!(
        solver.solve(&model, &HighsOptions::default()),
        Err(SolverError::UnsupportedConstraint("SOS1/SOS2"))
    ));
}

#[test]
fn highs_persistent_rejects_sos_constraints() {
    let model = sos_model();
    let mut solver = Highs.persistent();
    assert!(matches!(
        solver.solve(&model, &HighsOptions::default()),
        Err(SolverError::UnsupportedConstraint("SOS1/SOS2"))
    ));
}
