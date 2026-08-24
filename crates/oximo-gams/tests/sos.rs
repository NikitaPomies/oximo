use oximo_core::prelude::*;
use oximo_gams::Gams;
use oximo_solver::Solver;

#[test]
fn gams_advertises_sos1_and_sos2_support() {
    let model = Model::new("gams_sos");
    variable!(model, x);
    variable!(model, y);
    sos_constraint!(model, choice, SOS1, [(x, 1.0), (y, 2.0)]);
    sos_constraint!(model, adjacent, SOS2, [(x, 1.0), (y, 2.0)]);

    let solver = Gams::new();
    assert!(solver.supports_sos(SosType::Sos1));
    assert!(solver.supports_sos(SosType::Sos2));
    assert!(solver.supports_model(&model));
}
