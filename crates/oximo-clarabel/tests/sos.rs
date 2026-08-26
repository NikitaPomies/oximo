use oximo_clarabel::{Clarabel, ClarabelOptions};
use oximo_core::prelude::*;
use oximo_solver::{PersistentSolver, Solver, SolverError};

fn sos_model() -> Model {
    let m = Model::new("clarabel_sos");
    variable!(m, x);
    variable!(m, y);
    sos_constraint!(m, choice, SOS1, [x, y]);
    m
}

#[test]
fn clarabel_rejects_sos_constraints() {
    let model = sos_model();
    let mut solver = Clarabel;
    assert!(matches!(
        solver.solve(&model, &ClarabelOptions::default()),
        Err(SolverError::UnsupportedSos)
    ));
}

#[test]
fn clarabel_persistent_rejects_sos_constraints() {
    let model = sos_model();
    let mut solver = Clarabel.persistent();
    assert!(matches!(
        solver.solve(&model, &ClarabelOptions::default()),
        Err(SolverError::UnsupportedSos)
    ));
}
