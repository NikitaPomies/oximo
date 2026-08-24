use oximo_baron::{Baron, BaronOptions};
use oximo_core::prelude::*;
use oximo_solver::{Solver, SolverError};

#[test]
fn baron_rejects_sos_constraints_before_launching_executable() {
    let model = Model::new("baron_sos");
    variable!(model, x);
    variable!(model, y);
    sos_constraint!(model, choice, SOS1, [x, y]);
    let mut solver = Baron::with_exec("definitely-not-a-baron");
    assert!(matches!(
        solver.solve(&model, &BaronOptions::default()),
        Err(SolverError::UnsupportedConstraint("SOS1/SOS2"))
    ));
}
