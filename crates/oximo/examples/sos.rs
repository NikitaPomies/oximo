//! Native SOS1 and SOS2 constraints with Gurobi.
//!
//! Run with:
//! ```text
//! cargo run -p oximo --example sos --features gurobi
//! ```

use oximo::prelude::*;
use oximo::{GurobiOptions, solvers::Gurobi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("sos_example");

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

    let result = Gurobi.solve(&model, &GurobiOptions::default())?;
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
