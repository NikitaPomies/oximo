use oximo_core::prelude::*;
use oximo_gurobi::{Gurobi, GurobiOptions};
use oximo_solver::Solver;

#[test]
fn native_sos1_and_sos2_solve() {
    let m = Model::new("sos");
    variable!(m, 0.0 <= x <= 10.0);
    variable!(m, 0.0 <= y <= 20.0);
    variable!(m, 0.0 <= z <= 30.0);
    objective!(m, Max, x + y + z);
    sos_constraint!(m, one, SOS1, [(x, 1.0), (y, 2.0)]);
    sos_constraint!(m, two, SOS2, [(x, 1.0), (y, 2.0), (z, 3.0)]);
    assert!(Gurobi.supports_model(&m));
    let result = Gurobi.solve(&m, &GurobiOptions::default()).expect("Gurobi SOS solve");
    assert!(result.has_solution());
    assert_eq!(m.kind(), ModelKind::MILP);
    assert!(result.value_of(y).unwrap() > 19.0);
    assert!(result.value_of(x).unwrap().abs() < 1e-6);
}
