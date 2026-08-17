//! Live callback-surface tests for the Gurobi 13 backend.

use oximo_core::prelude::*;
use oximo_gurobi::{
    Callback, CallbackLocation, CallbackMask, CbResult, Gurobi, GurobiOptions, GurobiPresolve,
    Where,
};

#[derive(Default)]
struct Locations(Vec<CallbackLocation>);

impl Callback for Locations {
    fn callback(&mut self, where_: Where) -> CbResult {
        let location = match where_ {
            Where::Polling(_) => CallbackLocation::Polling,
            Where::PreSolve(_) => CallbackLocation::PreSolve,
            Where::Simplex(_) => CallbackLocation::Simplex,
            Where::MIP(_) => CallbackLocation::Mip,
            Where::MIPSol(_) => CallbackLocation::MipSol,
            Where::MIPNode(_) => CallbackLocation::MipNode,
            Where::Message(_) => CallbackLocation::Message,
            Where::Barrier(_) => CallbackLocation::Barrier,
            Where::MultiObj(_) => CallbackLocation::MultiObj,
            Where::IIS(_) => CallbackLocation::Iis,
            _ => return Err(std::io::Error::other("unexpected callback location").into()),
        };
        self.0.push(location);
        Ok(())
    }
}

#[test]
fn callback_solve_uses_common_result_collection() {
    let m = Model::new("callback");
    variable!(m, x[i in 0..4], Bin);
    constraint!(m, cap, sum!(x[i] for i in 0..4) <= 2.0);
    objective!(m, Max, sum!(x[i] for i in 0..4));

    let mut callback = Locations::default();
    let mut solver = Gurobi;
    let mask = CallbackMask::from(CallbackLocation::Polling) | CallbackLocation::Mip;
    let result = solver
        .solve_with_callback_filtered(
            &m,
            &GurobiOptions::default().presolve(GurobiPresolve::Off),
            &mut callback,
            mask,
        )
        .expect("callback solve");

    assert!(result.has_solution());
    assert!(!callback.0.is_empty(), "Gurobi did not invoke the callback");
    for location in callback.0 {
        assert!(mask.contains(location), "callback location {location:?} was not selected");
    }
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
