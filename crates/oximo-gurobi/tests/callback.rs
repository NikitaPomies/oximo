//! Live callback-surface tests for the Gurobi 13 backend.

use oximo_core::prelude::*;
use oximo_gurobi::{
    Callback, CallbackLocation, CallbackMask, CbResult, Gurobi, GurobiOptions, Where,
};

#[derive(Default)]
struct Counter(usize);

impl Callback for Counter {
    fn callback(&mut self, _where: Where) -> CbResult {
        self.0 += 1;
        Ok(())
    }
}

#[test]
fn callback_solve_uses_common_result_collection() {
    let m = Model::new("callback");
    variable!(m, x[i in 0..4], Bin);
    constraint!(m, cap, sum!(x[i] for i in 0..4) <= 2.0);
    objective!(m, Max, sum!(x[i] for i in 0..4));

    let mut callback = Counter::default();
    let mut solver = Gurobi;
    let result = solver
        .solve_with_callback_filtered(
            &m,
            &GurobiOptions::default(),
            &mut callback,
            CallbackMask::from(CallbackLocation::Polling) | CallbackLocation::Mip,
        )
        .expect("callback solve");

    assert!(result.has_solution());
    assert!(callback.0 > 0, "Gurobi did not invoke the callback");
}

struct FailingCallback;

impl Callback for FailingCallback {
    fn callback(&mut self, _where: Where) -> CbResult {
        Err(std::io::Error::other("callback failure").into())
    }
}

#[test]
fn callback_errors_are_solver_errors() {
    let m = Model::new("callback_error");
    variable!(m, x, Bin);
    objective!(m, Max, x);

    let mut callback = FailingCallback;
    let mut solver = Gurobi;
    let error = solver
        .solve_with_callback(&m, &GurobiOptions::default(), &mut callback)
        .expect_err("callback should fail");
    assert!(error.to_string().contains("Problem with callback"), "{error}");
}
