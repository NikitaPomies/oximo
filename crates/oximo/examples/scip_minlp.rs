//! A small mixed-integer nonlinear program solved by SCIP.
//!
//! The integer variable `x` selects a discrete production level while the
//! continuous variable `y` is limited by an exponential process law.
//!
//! ```text
//! maximize       3 x + y
//! subject to     x^2 + exp(y) ≤ 12
//!                0 <= x <= 3, x integer
//!                0 <= y <= 3
//! ```
//!
//! With the bundled backend:
//!
//! ```text
//! cargo run -p oximo --no-default-features --features scip --example scip_minlp
//! ```
//!
//! To use a local SCIP installation, provide its development tree (including
//! headers and `libscip`) through `SCIPOPTDIR` and select `scip-system`:
//!
//! ```text
//! SCIPOPTDIR=/path/to/scip cargo run -p oximo --no-default-features \
//!   --features scip-system --example scip_minlp
//! ```

#[cfg(any(feature = "scip", feature = "scip-system"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use oximo::ScipOptions;
    use oximo::prelude::*;
    use oximo::solvers::Scip;

    let model = Model::new("scip_minlp");
    variable!(model, 0.0 <= x <= 3.0, Int);
    variable!(model, 0.0 <= y <= 3.0);

    constraint!(model, process_limit, x.powi(2) + y.exp() <= 12.0);
    objective!(model, Max, 3.0 * x + y);

    assert_eq!(model.kind(), ModelKind::MINLP);

    let mut solver = Scip::new();
    let output = solver.solve_detailed(&model, &ScipOptions::default())?;
    let result = output.result;
    assert!(result.has_solution());
    let solution = result.best().ok_or("SCIP returned no incumbent")?;
    let x_value = solution.value_of(x)?.ok_or("SCIP did not return x")?;
    let y_value = solution.value_of(y)?.ok_or("SCIP did not return y")?;
    let objective = result.objective().ok_or("SCIP did not return an objective")?;

    println!("SCIP MINLP example");
    println!("  termination: {:?}", result.termination);
    println!("  x (integer): {x_value:.6}");
    println!("  y          : {y_value:.6}");
    println!("  objective  : {objective:.6}");
    println!("  nodes      : {:?}", result.node_count);
    println!("  SCIP stats : {} bytes of JSON", output.statistics_json.len());

    // The global optimum is x = 3 and y = ln(3), up to SCIP's feasibility
    // tolerance.
    assert!((x_value - 3.0).abs() <= 1e-5);
    assert!((y_value - 3.0_f64.ln()).abs() <= 1e-4);
    assert!((objective - (9.0 + 3.0_f64.ln())).abs() <= 1e-4);

    Ok(())
}

#[cfg(not(any(feature = "scip", feature = "scip-system")))]
fn main() {
    println!("Enable a SCIP backend feature:");
    println!("  cargo run -p oximo --no-default-features --features scip --example scip_minlp");
    println!(
        "  cargo run -p oximo --no-default-features --features scip-system --example scip_minlp"
    );
}
