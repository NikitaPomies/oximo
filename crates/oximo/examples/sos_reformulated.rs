//! SOS1 and SOS2 constraints explicitly reformulated and solved with HiGHS.
//!
//! This is the portable-MILP counterpart to the native-Gurobi `sos` example.
//!
//! Run with:
//! ```text
//! cargo run -p oximo --example sos_reformulated --features highs
//! ```

use oximo::prelude::*;
use oximo::{HighsOptions, solvers::Highs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("sos_reformulated_example");

    variable!(model, 0.0 <= choice_a <= 10.0);
    variable!(model, 0.0 <= choice_b <= 10.0);
    variable!(model, 0.0 <= choice_c <= 10.0);
    variable!(model, 0.0 <= curve_0 <= 10.0);
    variable!(model, 0.0 <= curve_1 <= 10.0);
    variable!(model, 0.0 <= curve_2 <= 10.0);

    // At most one choice variable may be nonzero.
    sos_constraint!(model, one_choice, SOS1, [choice_a, choice_b, choice_c]);

    // At most two curve variables may be nonzero, and they must be adjacent
    // according to their weights.
    sos_constraint!(model, adjacent_curve, SOS2, [(curve_0, 1.0), (curve_1, 2.0), (curve_2, 3.0),]);

    objective!(
        model,
        Max,
        choice_a + 2.0 * choice_b + 3.0 * choice_c + curve_0 + curve_1 + curve_2
    );

    // HiGHS has no native SOS support, so explicitly replace every active SOS
    // with binary variables and linear Big-M rows.
    //
    // The finite member bounds are used as their individual Big-M values.
    let reformulations = model.reformulate_sos(SosReformulationOptions::default())?;
    println!("reformulated {} SOS constraints", reformulations.len());

    let result = Highs.solve(&model, &HighsOptions::default())?;
    println!("termination = {:?}", result.termination);
    for (name, value) in [
        ("choice_a", result.value_of(choice_a)),
        ("choice_b", result.value_of(choice_b)),
        ("choice_c", result.value_of(choice_c)),
        ("curve_0", result.value_of(curve_0)),
        ("curve_1", result.value_of(curve_1)),
        ("curve_2", result.value_of(curve_2)),
    ] {
        println!("{name:>8} = {:.6}", value.unwrap_or_default());
    }
    Ok(())
}
