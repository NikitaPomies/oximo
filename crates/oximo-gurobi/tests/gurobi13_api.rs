use oximo_gurobi::{GRB_METHOD_PDHG, GurobiOptions, Opcode, Status};

#[test]
fn exposes_gurobi13_symbols_and_method_option() {
    let _options = GurobiOptions::default()
        .method(GRB_METHOD_PDHG)
        .nl_bar_iter_limit(100)
        .nl_bar_c_feas_tol(1e-6)
        .nl_bar_d_feas_tol(1e-6)
        .nl_bar_p_feas_tol(1e-6)
        .pdhg_abs_tol(1e-6)
        .pdhg_conv_tol(1e-6)
        .pdhg_rel_tol(1e-6)
        .pdhg_iter_limit(1000.0)
        .pdhg_gpu(1)
        .no_rel_heur_solutions(10)
        .inherit_params(1)
        .fix_vars_in_indicators(1)
        .start_time_limit(1.0)
        .start_work_limit(1.0)
        .improve_start_work(1.0)
        .master_knapsack_cuts(1)
        .obj_pass_number(1)
        .optimality_target(1);
    assert_eq!(Status::LocallyOptimal as i32, 18);
    assert_eq!(Status::LocallyInfeasible as i32, 19);
    assert_eq!(Opcode::SignPow as i32, 19);
}
