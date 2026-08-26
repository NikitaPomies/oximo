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
        Err(SolverError::UnsupportedSos)
    ));
}

#[test]
fn highs_persistent_rejects_sos_constraints() {
    let model = sos_model();
    let mut solver = Highs.persistent();
    assert!(matches!(
        solver.solve(&model, &HighsOptions::default()),
        Err(SolverError::UnsupportedSos)
    ));
}

#[test]
fn highs_solves_explicit_sos1_reformulation() {
    let model = Model::new("highs_reformulated_sos1");
    variable!(model, 0.0 <= x <= 1.0);
    variable!(model, 0.0 <= y <= 1.0);
    objective!(model, Max, x + y);
    sos_constraint!(model, choice, SOS1, [x, y]);
    let transformed = model.to_reformulated_sos_model(SosReformulationOptions::default()).unwrap();

    let mut solver = Highs;
    let result = solver.solve(&transformed, &HighsOptions::default()).unwrap();
    assert!((result.objective().unwrap() - 1.0).abs() < 1e-7);
    assert!(result.value_of(x).unwrap() + result.value_of(y).unwrap() <= 1.0 + 1e-7);
}

#[test]
fn highs_solves_weight_ordered_sos2_reformulation() {
    let model = Model::new("highs_reformulated_sos2");
    variable!(model, 0.0 <= x <= 1.0);
    variable!(model, 0.0 <= y <= 1.0);
    variable!(model, 0.0 <= z <= 1.0);
    objective!(model, Max, x + z);
    sos_constraint!(model, adjacent, SOS2, [(z, 3.0), (x, 1.0), (y, 2.0)]);
    let transformed = model.to_reformulated_sos_model(SosReformulationOptions::default()).unwrap();

    let mut solver = Highs;
    let result = solver.solve(&transformed, &HighsOptions::default()).unwrap();
    assert!((result.objective().unwrap() - 1.0).abs() < 1e-7);
    assert!(result.value_of(x).unwrap() + result.value_of(z).unwrap() <= 1.0 + 1e-7);
}
