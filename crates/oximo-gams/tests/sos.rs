use oximo_core::prelude::*;
use oximo_gams::{Gams, GamsOptions};
use oximo_solver::{Solver, SolverError};

#[test]
fn gams_rejects_sos_constraints_before_launching_executable() {
    let model = Model::new("gams_sos");
    variable!(model, x);
    variable!(model, y);
    sos_constraint!(model, choice, SOS1, [x, y]);
    let mut solver = Gams::with_exec("definitely-not-a-gams");
    assert!(matches!(
        solver.solve(&model, &GamsOptions::default()),
        Err(SolverError::UnsupportedConstraint("SOS1/SOS2"))
    ));
}
