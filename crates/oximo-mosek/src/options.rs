use mosek::{Dparam, Iparam, Sparam, Task};
use oximo_solver::{HasUniversal, SolverError, UniversalOptions};

/// MOSEK-specific solver options.
///
/// Universal options are available through
/// [`UniversalOptionsExt`](oximo_solver::UniversalOptionsExt).
/// The typed methods below cover every parameter in the MOSEK 11.2
/// Rust binding.
#[derive(Clone, Debug, Default)]
pub struct MosekOptions {
    pub universal: UniversalOptions,
    double_params: Vec<(i32, f64)>,
    int_params: Vec<(i32, i32)>,
    string_params: Vec<(i32, String)>,
}

impl HasUniversal for MosekOptions {
    fn universal(&self) -> &UniversalOptions {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalOptions {
        &mut self.universal
    }
}

macro_rules! mosek_params {
    ($(($kind:ident, $method:ident, $variant:ident)),* $(,)?) => {
        $(mosek_params!(@method $kind, $method, $variant);)*
        #[cfg(test)]
        pub(crate) const DOUBLE_PARAM_IDS: &[Option<i32>] =
            &[$(mosek_params!(@id dbl, $kind, $variant)),*];
        #[cfg(test)]
        pub(crate) const INT_PARAM_IDS: &[Option<i32>] =
            &[$(mosek_params!(@id int, $kind, $variant)),*];
        #[cfg(test)]
        pub(crate) const STRING_PARAM_IDS: &[Option<i32>] =
            &[$(mosek_params!(@id str, $kind, $variant)),*];
    };
    (@method dbl, $method:ident, $variant:ident) => {
        #[doc = concat!(
            "Sets MOSEK double parameter [`",
            stringify!($variant),
            "`](mosek::Dparam::",
            stringify!($variant),
            "). The linked constant carries MOSEK's upstream parameter description."
        )]
        #[must_use]
        pub fn $method(mut self, value: f64) -> Self {
            self.double_params.push((Dparam::$variant, value));
            self
        }
    };
    (@method int, $method:ident, $variant:ident) => {
        #[doc = concat!(
            "Sets MOSEK integer parameter [`",
            stringify!($variant),
            "`](mosek::Iparam::",
            stringify!($variant),
            "). The linked constant carries MOSEK's upstream parameter description."
        )]
        #[must_use]
        pub fn $method(mut self, value: i32) -> Self {
            self.int_params.push((Iparam::$variant, value));
            self
        }
    };
    (@method str, $method:ident, $variant:ident) => {
        #[doc = concat!(
            "Sets MOSEK string parameter [`",
            stringify!($variant),
            "`](mosek::Sparam::",
            stringify!($variant),
            "). The linked constant carries MOSEK's upstream parameter description."
        )]
        #[must_use]
        pub fn $method(mut self, value: impl Into<String>) -> Self {
            self.string_params.push((Sparam::$variant, value.into()));
            self
        }
    };
    (@id dbl, dbl, $variant:ident) => { Some(Dparam::$variant) };
    (@id dbl, $other:ident, $variant:ident) => { None };
    (@id int, int, $variant:ident) => { Some(Iparam::$variant) };
    (@id int, $other:ident, $variant:ident) => { None };
    (@id str, str, $variant:ident) => { Some(Sparam::$variant) };
    (@id str, $other:ident, $variant:ident) => { None };
}

impl MosekOptions {
    mosek_params!(
        (dbl, ana_sol_infeas_tol, ANA_SOL_INFEAS_TOL),
        (dbl, basis_rel_tol_s, BASIS_REL_TOL_S),
        (dbl, basis_tol_s, BASIS_TOL_S),
        (dbl, basis_tol_x, BASIS_TOL_X),
        (dbl, data_sym_mat_tol, DATA_SYM_MAT_TOL),
        (dbl, data_sym_mat_tol_huge, DATA_SYM_MAT_TOL_HUGE),
        (dbl, data_sym_mat_tol_large, DATA_SYM_MAT_TOL_LARGE),
        (dbl, data_tol_aij_huge, DATA_TOL_AIJ_HUGE),
        (dbl, data_tol_aij_large, DATA_TOL_AIJ_LARGE),
        (dbl, data_tol_bound_inf, DATA_TOL_BOUND_INF),
        (dbl, data_tol_bound_wrn, DATA_TOL_BOUND_WRN),
        (dbl, data_tol_c_huge, DATA_TOL_C_HUGE),
        (dbl, data_tol_cj_large, DATA_TOL_CJ_LARGE),
        (dbl, data_tol_qij, DATA_TOL_QIJ),
        (dbl, data_tol_x, DATA_TOL_X),
        (dbl, folding_tol_eq, FOLDING_TOL_EQ),
        (dbl, intpnt_co_tol_dfeas, INTPNT_CO_TOL_DFEAS),
        (dbl, intpnt_co_tol_infeas, INTPNT_CO_TOL_INFEAS),
        (dbl, intpnt_co_tol_mu_red, INTPNT_CO_TOL_MU_RED),
        (dbl, intpnt_co_tol_near_rel, INTPNT_CO_TOL_NEAR_REL),
        (dbl, intpnt_co_tol_pfeas, INTPNT_CO_TOL_PFEAS),
        (dbl, intpnt_co_tol_rel_gap, INTPNT_CO_TOL_REL_GAP),
        (dbl, intpnt_qo_tol_dfeas, INTPNT_QO_TOL_DFEAS),
        (dbl, intpnt_qo_tol_infeas, INTPNT_QO_TOL_INFEAS),
        (dbl, intpnt_qo_tol_mu_red, INTPNT_QO_TOL_MU_RED),
        (dbl, intpnt_qo_tol_near_rel, INTPNT_QO_TOL_NEAR_REL),
        (dbl, intpnt_qo_tol_pfeas, INTPNT_QO_TOL_PFEAS),
        (dbl, intpnt_qo_tol_rel_gap, INTPNT_QO_TOL_REL_GAP),
        (dbl, intpnt_tol_dfeas, INTPNT_TOL_DFEAS),
        (dbl, intpnt_tol_dsafe, INTPNT_TOL_DSAFE),
        (dbl, intpnt_tol_infeas, INTPNT_TOL_INFEAS),
        (dbl, intpnt_tol_mu_red, INTPNT_TOL_MU_RED),
        (dbl, intpnt_tol_path, INTPNT_TOL_PATH),
        (dbl, intpnt_tol_pfeas, INTPNT_TOL_PFEAS),
        (dbl, intpnt_tol_psafe, INTPNT_TOL_PSAFE),
        (dbl, intpnt_tol_rel_gap, INTPNT_TOL_REL_GAP),
        (dbl, intpnt_tol_rel_step, INTPNT_TOL_REL_STEP),
        (dbl, intpnt_tol_step_size, INTPNT_TOL_STEP_SIZE),
        (dbl, lower_obj_cut, LOWER_OBJ_CUT),
        (dbl, lower_obj_cut_finite_trh, LOWER_OBJ_CUT_FINITE_TRH),
        (dbl, mio_clique_table_size_factor, MIO_CLIQUE_TABLE_SIZE_FACTOR),
        (dbl, mio_djc_max_bigm, MIO_DJC_MAX_BIGM),
        (dbl, mio_max_time, MIO_MAX_TIME),
        (dbl, mio_rel_gap_const, MIO_REL_GAP_CONST),
        (dbl, mio_tol_abs_gap, MIO_TOL_ABS_GAP),
        (dbl, mio_tol_abs_relax_int, MIO_TOL_ABS_RELAX_INT),
        (dbl, mio_tol_feas, MIO_TOL_FEAS),
        (dbl, mio_tol_rel_dual_bound_improvement, MIO_TOL_REL_DUAL_BOUND_IMPROVEMENT),
        (dbl, mio_tol_rel_gap, MIO_TOL_REL_GAP),
        (dbl, optimizer_max_ticks, OPTIMIZER_MAX_TICKS),
        (dbl, optimizer_max_time, OPTIMIZER_MAX_TIME),
        (dbl, presolve_tol_abs_lindep, PRESOLVE_TOL_ABS_LINDEP),
        (dbl, presolve_tol_primal_infeas_perturbation, PRESOLVE_TOL_PRIMAL_INFEAS_PERTURBATION),
        (dbl, presolve_tol_rel_lindep, PRESOLVE_TOL_REL_LINDEP),
        (dbl, presolve_tol_s, PRESOLVE_TOL_S),
        (dbl, presolve_tol_x, PRESOLVE_TOL_X),
        (dbl, qcqo_reformulate_rel_drop_tol, QCQO_REFORMULATE_REL_DROP_TOL),
        (dbl, semidefinite_tol_approx, SEMIDEFINITE_TOL_APPROX),
        (dbl, sim_lu_tol_rel_piv, SIM_LU_TOL_REL_PIV),
        (dbl, sim_precision_scaling_extended, SIM_PRECISION_SCALING_EXTENDED),
        (dbl, sim_precision_scaling_normal, SIM_PRECISION_SCALING_NORMAL),
        (dbl, simplex_abs_tol_piv, SIMPLEX_ABS_TOL_PIV),
        (dbl, upper_obj_cut, UPPER_OBJ_CUT),
        (dbl, upper_obj_cut_finite_trh, UPPER_OBJ_CUT_FINITE_TRH),
        (int, ana_sol_basis, ANA_SOL_BASIS),
        (int, ana_sol_print_violated, ANA_SOL_PRINT_VIOLATED),
        (int, auto_sort_a_before_opt, AUTO_SORT_A_BEFORE_OPT),
        (int, auto_update_sol_info, AUTO_UPDATE_SOL_INFO),
        (int, basis_solve_use_plus_one, BASIS_SOLVE_USE_PLUS_ONE),
        (int, bi_clean_optimizer, BI_CLEAN_OPTIMIZER),
        (int, bi_ignore_max_iter, BI_IGNORE_MAX_ITER),
        (int, bi_ignore_num_error, BI_IGNORE_NUM_ERROR),
        (int, bi_max_iterations, BI_MAX_ITERATIONS),
        (int, cache_license, CACHE_LICENSE),
        (int, compress_statfile, COMPRESS_STATFILE),
        (int, folding_use, FOLDING_USE),
        (int, getdual_convert_lmis, GETDUAL_CONVERT_LMIS),
        (int, heartbeat_sim_freq_ticks, HEARTBEAT_SIM_FREQ_TICKS),
        (int, infeas_generic_names, INFEAS_GENERIC_NAMES),
        (int, infeas_report_auto, INFEAS_REPORT_AUTO),
        (int, infeas_report_level, INFEAS_REPORT_LEVEL),
        (int, intpnt_basis, INTPNT_BASIS),
        (int, intpnt_diff_step, INTPNT_DIFF_STEP),
        (int, intpnt_hotstart, INTPNT_HOTSTART),
        (int, intpnt_max_iterations, INTPNT_MAX_ITERATIONS),
        (int, intpnt_max_num_cor, INTPNT_MAX_NUM_COR),
        (int, intpnt_off_col_trh, INTPNT_OFF_COL_TRH),
        (int, intpnt_order_gp_num_seeds, INTPNT_ORDER_GP_NUM_SEEDS),
        (int, intpnt_order_method, INTPNT_ORDER_METHOD),
        (int, intpnt_regularization_use, INTPNT_REGULARIZATION_USE),
        (int, intpnt_scaling, INTPNT_SCALING),
        (int, intpnt_solve_form, INTPNT_SOLVE_FORM),
        (int, intpnt_starting_point, INTPNT_STARTING_POINT),
        (int, license_debug, LICENSE_DEBUG),
        (int, license_pause_time, LICENSE_PAUSE_TIME),
        (int, license_suppress_expire_wrns, LICENSE_SUPPRESS_EXPIRE_WRNS),
        (int, license_trh_expiry_wrn, LICENSE_TRH_EXPIRY_WRN),
        (int, license_wait, LICENSE_WAIT),
        (int, log, LOG),
        (int, log_ana_pro, LOG_ANA_PRO),
        (int, log_bi, LOG_BI),
        (int, log_bi_freq, LOG_BI_FREQ),
        (int, log_cut_second_opt, LOG_CUT_SECOND_OPT),
        (int, log_expand, LOG_EXPAND),
        (int, log_feas_repair, LOG_FEAS_REPAIR),
        (int, log_file, LOG_FILE),
        (int, log_include_summary, LOG_INCLUDE_SUMMARY),
        (int, log_infeas_ana, LOG_INFEAS_ANA),
        (int, log_intpnt, LOG_INTPNT),
        (int, log_local_info, LOG_LOCAL_INFO),
        (int, log_mio, LOG_MIO),
        (int, log_mio_freq, LOG_MIO_FREQ),
        (int, log_order, LOG_ORDER),
        (int, log_presolve, LOG_PRESOLVE),
        (int, log_sensitivity, LOG_SENSITIVITY),
        (int, log_sensitivity_opt, LOG_SENSITIVITY_OPT),
        (int, log_sim, LOG_SIM),
        (int, log_sim_freq, LOG_SIM_FREQ),
        (int, log_sim_freq_giga_ticks, LOG_SIM_FREQ_GIGA_TICKS),
        (int, log_storage, LOG_STORAGE),
        (int, max_num_warnings, MAX_NUM_WARNINGS),
        (int, mio_branch_dir, MIO_BRANCH_DIR),
        (int, mio_conflict_analysis_level, MIO_CONFLICT_ANALYSIS_LEVEL),
        (int, mio_conic_outer_approximation, MIO_CONIC_OUTER_APPROXIMATION),
        (int, mio_construct_sol, MIO_CONSTRUCT_SOL),
        (int, mio_crossover_max_nodes, MIO_CROSSOVER_MAX_NODES),
        (int, mio_cut_clique, MIO_CUT_CLIQUE),
        (int, mio_cut_cmir, MIO_CUT_CMIR),
        (int, mio_cut_gmi, MIO_CUT_GMI),
        (int, mio_cut_implied_bound, MIO_CUT_IMPLIED_BOUND),
        (int, mio_cut_knapsack_cover, MIO_CUT_KNAPSACK_COVER),
        (int, mio_cut_lipro, MIO_CUT_LIPRO),
        (int, mio_cut_selection_level, MIO_CUT_SELECTION_LEVEL),
        (int, mio_data_permutation_method, MIO_DATA_PERMUTATION_METHOD),
        (int, mio_dual_ray_analysis_level, MIO_DUAL_RAY_ANALYSIS_LEVEL),
        (int, mio_feaspump_level, MIO_FEASPUMP_LEVEL),
        (int, mio_heuristic_level, MIO_HEURISTIC_LEVEL),
        (int, mio_independent_block_level, MIO_INDEPENDENT_BLOCK_LEVEL),
        (int, mio_max_num_branches, MIO_MAX_NUM_BRANCHES),
        (int, mio_max_num_relaxs, MIO_MAX_NUM_RELAXS),
        (int, mio_max_num_restarts, MIO_MAX_NUM_RESTARTS),
        (int, mio_max_num_root_cut_rounds, MIO_MAX_NUM_ROOT_CUT_ROUNDS),
        (int, mio_max_num_solutions, MIO_MAX_NUM_SOLUTIONS),
        (int, mio_memory_emphasis_level, MIO_MEMORY_EMPHASIS_LEVEL),
        (int, mio_min_rel, MIO_MIN_REL),
        (int, mio_mode, MIO_MODE),
        (int, mio_node_optimizer, MIO_NODE_OPTIMIZER),
        (int, mio_node_selection, MIO_NODE_SELECTION),
        (int, mio_numerical_emphasis_level, MIO_NUMERICAL_EMPHASIS_LEVEL),
        (int, mio_opt_face_max_nodes, MIO_OPT_FACE_MAX_NODES),
        (int, mio_perspective_reformulate, MIO_PERSPECTIVE_REFORMULATE),
        (int, mio_presolve_aggregator_use, MIO_PRESOLVE_AGGREGATOR_USE),
        (int, mio_probing_level, MIO_PROBING_LEVEL),
        (int, mio_propagate_objective_constraint, MIO_PROPAGATE_OBJECTIVE_CONSTRAINT),
        (int, mio_qcqo_reformulation_method, MIO_QCQO_REFORMULATION_METHOD),
        (int, mio_rens_max_nodes, MIO_RENS_MAX_NODES),
        (int, mio_rins_max_nodes, MIO_RINS_MAX_NODES),
        (int, mio_root_optimizer, MIO_ROOT_OPTIMIZER),
        (int, mio_seed, MIO_SEED),
        (int, mio_symmetry_level, MIO_SYMMETRY_LEVEL),
        (int, mio_var_selection, MIO_VAR_SELECTION),
        (int, mio_vb_detection_level, MIO_VB_DETECTION_LEVEL),
        (int, mt_spincount, MT_SPINCOUNT),
        (int, ng, NG),
        (int, num_threads, NUM_THREADS),
        (int, opf_write_header, OPF_WRITE_HEADER),
        (int, opf_write_hints, OPF_WRITE_HINTS),
        (int, opf_write_line_length, OPF_WRITE_LINE_LENGTH),
        (int, opf_write_parameters, OPF_WRITE_PARAMETERS),
        (int, opf_write_problem, OPF_WRITE_PROBLEM),
        (int, opf_write_sol_bas, OPF_WRITE_SOL_BAS),
        (int, opf_write_sol_itg, OPF_WRITE_SOL_ITG),
        (int, opf_write_sol_itr, OPF_WRITE_SOL_ITR),
        (int, opf_write_solutions, OPF_WRITE_SOLUTIONS),
        (int, optimizer, OPTIMIZER),
        (int, param_read_case_name, PARAM_READ_CASE_NAME),
        (int, param_read_ign_error, PARAM_READ_IGN_ERROR),
        (int, presolve_eliminator_max_fill, PRESOLVE_ELIMINATOR_MAX_FILL),
        (int, presolve_eliminator_max_num_tries, PRESOLVE_ELIMINATOR_MAX_NUM_TRIES),
        (int, presolve_lindep_abs_work_trh, PRESOLVE_LINDEP_ABS_WORK_TRH),
        (int, presolve_lindep_new, PRESOLVE_LINDEP_NEW),
        (int, presolve_lindep_rel_work_trh, PRESOLVE_LINDEP_REL_WORK_TRH),
        (int, presolve_lindep_use, PRESOLVE_LINDEP_USE),
        (int, presolve_max_num_pass, PRESOLVE_MAX_NUM_PASS),
        (int, presolve_max_num_reductions, PRESOLVE_MAX_NUM_REDUCTIONS),
        (int, presolve_use, PRESOLVE_USE),
        (int, primal_repair_optimizer, PRIMAL_REPAIR_OPTIMIZER),
        (int, ptf_write_parameters, PTF_WRITE_PARAMETERS),
        (int, ptf_write_single_psd_terms, PTF_WRITE_SINGLE_PSD_TERMS),
        (int, ptf_write_solutions, PTF_WRITE_SOLUTIONS),
        (int, ptf_write_transform, PTF_WRITE_TRANSFORM),
        (int, read_async, READ_ASYNC),
        (int, read_debug, READ_DEBUG),
        (int, read_keep_free_con, READ_KEEP_FREE_CON),
        (int, read_mps_format, READ_MPS_FORMAT),
        (int, read_mps_width, READ_MPS_WIDTH),
        (int, read_task_ignore_param, READ_TASK_IGNORE_PARAM),
        (int, remote_use_compression, REMOTE_USE_COMPRESSION),
        (int, remove_unused_solutions, REMOVE_UNUSED_SOLUTIONS),
        (int, sensitivity_all, SENSITIVITY_ALL),
        (int, sensitivity_type, SENSITIVITY_TYPE),
        (int, sim_basis_factor_use, SIM_BASIS_FACTOR_USE),
        (int, sim_degen, SIM_DEGEN),
        (int, sim_detect_pwl, SIM_DETECT_PWL),
        (int, sim_dual_crash, SIM_DUAL_CRASH),
        (int, sim_dual_phaseone_method, SIM_DUAL_PHASEONE_METHOD),
        (int, sim_dual_restrict_selection, SIM_DUAL_RESTRICT_SELECTION),
        (int, sim_dual_selection, SIM_DUAL_SELECTION),
        (int, sim_exploit_dupvec, SIM_EXPLOIT_DUPVEC),
        (int, sim_hotstart, SIM_HOTSTART),
        (int, sim_hotstart_lu, SIM_HOTSTART_LU),
        (int, sim_max_iterations, SIM_MAX_ITERATIONS),
        (int, sim_max_num_setbacks, SIM_MAX_NUM_SETBACKS),
        (int, sim_non_singular, SIM_NON_SINGULAR),
        (int, sim_precision, SIM_PRECISION),
        (int, sim_precision_boost, SIM_PRECISION_BOOST),
        (int, sim_primal_crash, SIM_PRIMAL_CRASH),
        (int, sim_primal_phaseone_method, SIM_PRIMAL_PHASEONE_METHOD),
        (int, sim_primal_restrict_selection, SIM_PRIMAL_RESTRICT_SELECTION),
        (int, sim_primal_selection, SIM_PRIMAL_SELECTION),
        (int, sim_refactor_freq, SIM_REFACTOR_FREQ),
        (int, sim_reformulation, SIM_REFORMULATION),
        (int, sim_save_lu, SIM_SAVE_LU),
        (int, sim_scaling, SIM_SCALING),
        (int, sim_scaling_method, SIM_SCALING_METHOD),
        (int, sim_seed, SIM_SEED),
        (int, sim_solve_form, SIM_SOLVE_FORM),
        (int, sim_switch_optimizer, SIM_SWITCH_OPTIMIZER),
        (int, sol_filter_keep_basic, SOL_FILTER_KEEP_BASIC),
        (int, sol_read_name_width, SOL_READ_NAME_WIDTH),
        (int, sol_read_width, SOL_READ_WIDTH),
        (int, timing_level, TIMING_LEVEL),
        (int, write_async, WRITE_ASYNC),
        (int, write_bas_constraints, WRITE_BAS_CONSTRAINTS),
        (int, write_bas_head, WRITE_BAS_HEAD),
        (int, write_bas_variables, WRITE_BAS_VARIABLES),
        (int, write_compression, WRITE_COMPRESSION),
        (int, write_free_con, WRITE_FREE_CON),
        (int, write_generic_names, WRITE_GENERIC_NAMES),
        (int, write_ignore_incompatible_items, WRITE_IGNORE_INCOMPATIBLE_ITEMS),
        (int, write_int_constraints, WRITE_INT_CONSTRAINTS),
        (int, write_int_head, WRITE_INT_HEAD),
        (int, write_int_variables, WRITE_INT_VARIABLES),
        (int, write_json_indentation, WRITE_JSON_INDENTATION),
        (int, write_lp_full_obj, WRITE_LP_FULL_OBJ),
        (int, write_lp_line_width, WRITE_LP_LINE_WIDTH),
        (int, write_mps_format, WRITE_MPS_FORMAT),
        (int, write_mps_int, WRITE_MPS_INT),
        (int, write_sol_barvariables, WRITE_SOL_BARVARIABLES),
        (int, write_sol_constraints, WRITE_SOL_CONSTRAINTS),
        (int, write_sol_head, WRITE_SOL_HEAD),
        (int, write_sol_ignore_invalid_names, WRITE_SOL_IGNORE_INVALID_NAMES),
        (int, write_sol_variables, WRITE_SOL_VARIABLES),
        (str, bas_sol_file_name, BAS_SOL_FILE_NAME),
        (str, data_file_name, DATA_FILE_NAME),
        (str, debug_file_name, DEBUG_FILE_NAME),
        (str, int_sol_file_name, INT_SOL_FILE_NAME),
        (str, itr_sol_file_name, ITR_SOL_FILE_NAME),
        (str, mio_debug_string, MIO_DEBUG_STRING),
        (str, param_comment_sign, PARAM_COMMENT_SIGN),
        (str, param_read_file_name, PARAM_READ_FILE_NAME),
        (str, param_write_file_name, PARAM_WRITE_FILE_NAME),
        (str, read_mps_bou_name, READ_MPS_BOU_NAME),
        (str, read_mps_obj_name, READ_MPS_OBJ_NAME),
        (str, read_mps_ran_name, READ_MPS_RAN_NAME),
        (str, read_mps_rhs_name, READ_MPS_RHS_NAME),
        (str, remote_optserver_host, REMOTE_OPTSERVER_HOST),
        (str, remote_tls_cert, REMOTE_TLS_CERT),
        (str, remote_tls_cert_path, REMOTE_TLS_CERT_PATH),
        (str, sensitivity_file_name, SENSITIVITY_FILE_NAME),
        (str, sensitivity_res_file_name, SENSITIVITY_RES_FILE_NAME),
        (str, sol_filter_xc_low, SOL_FILTER_XC_LOW),
        (str, sol_filter_xc_upr, SOL_FILTER_XC_UPR),
        (str, sol_filter_xx_low, SOL_FILTER_XX_LOW),
        (str, sol_filter_xx_upr, SOL_FILTER_XX_UPR),
        (str, stat_key, STAT_KEY),
        (str, stat_name, STAT_NAME),
    );

    pub(crate) fn apply(&self, task: &mut Task) -> Result<(), SolverError> {
        if let Some(limit) = self.universal.time_limit {
            task.put_dou_param(Dparam::OPTIMIZER_MAX_TIME, limit.as_secs_f64()).map_err(backend)?;
        }
        if let Some(threads) = self.universal.threads {
            let threads = i32::try_from(threads)
                .map_err(|_| SolverError::Backend("MOSEK: thread count exceeds i32".into()))?;
            task.put_int_param(Iparam::NUM_THREADS, threads).map_err(backend)?;
        }
        if let Some(verbose) = self.universal.verbose {
            task.put_int_param(Iparam::LOG, i32::from(verbose)).map_err(backend)?;
        }
        for &(param, value) in &self.double_params {
            task.put_dou_param(param, value).map_err(backend)?;
        }
        for &(param, value) in &self.int_params {
            task.put_int_param(param, value).map_err(backend)?;
        }
        for (param, value) in &self.string_params {
            task.put_str_param(*param, value).map_err(backend)?;
        }
        Ok(())
    }
}

fn backend(message: String) -> SolverError {
    SolverError::Backend(format!("MOSEK: {message}"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use mosek::Parametertype;

    use super::*;

    fn assert_catalog(catalog: &[Option<i32>], expected: i32) {
        let ids: Vec<i32> = catalog.iter().flatten().copied().collect();
        assert_eq!(i32::try_from(ids.len()).unwrap(), expected);
        assert_eq!(ids.iter().copied().collect::<HashSet<_>>().len(), ids.len());
        assert_eq!(ids, (0..expected).collect::<Vec<_>>());
    }

    #[test]
    fn complete_parameter_catalog_matches_linked_library() {
        let task = Task::new().expect("create MOSEK task");
        let nd = task.get_num_param(Parametertype::DOU_TYPE).unwrap();
        let linked_ni = task.get_num_param(Parametertype::INT_TYPE).unwrap();
        let ns = task.get_num_param(Parametertype::STR_TYPE).unwrap();
        assert!(matches!(linked_ni, 64 | 189));
        let ni = 189;
        assert_catalog(MosekOptions::DOUBLE_PARAM_IDS, nd);
        assert_catalog(MosekOptions::INT_PARAM_IDS, ni);
        assert_catalog(MosekOptions::STRING_PARAM_IDS, ns);
        assert_eq!(nd + ni + ns, 277);
    }

    #[test]
    fn universal_and_backend_parameters_apply_in_precedence_order() {
        use std::time::Duration;

        use oximo_solver::UniversalOptionsExt;

        let opts = MosekOptions::default()
            .time_limit(Duration::from_secs(7))
            .threads(2)
            .verbose(true)
            .optimizer_max_time(3.5)
            .num_threads(4)
            .log(0)
            .mio_tol_rel_gap(1e-4)
            .remote_optserver_host("example.invalid");
        let mut task = Task::new().expect("create MOSEK task");
        opts.apply(&mut task).unwrap();
        assert!(
            (task.get_dou_param(Dparam::OPTIMIZER_MAX_TIME).unwrap() - 3.5).abs() < f64::EPSILON
        );
        assert_eq!(task.get_int_param(Iparam::NUM_THREADS).unwrap(), 4);
        assert_eq!(task.get_int_param(Iparam::LOG).unwrap(), 0);
        assert!((task.get_dou_param(Dparam::MIO_TOL_REL_GAP).unwrap() - 1e-4).abs() < f64::EPSILON);
        let mut len = task.get_str_param_len(Sparam::REMOTE_OPTSERVER_HOST).unwrap();
        assert_eq!(
            task.get_str_param(Sparam::REMOTE_OPTSERVER_HOST, &mut len).unwrap(),
            "example.invalid"
        );
    }

    #[test]
    fn repeated_backend_parameter_calls_are_retained() {
        let opts = MosekOptions::default()
            .mio_tol_rel_gap(0.1)
            .mio_tol_rel_gap(0.01)
            .presolve_use(mosek::Presolvemode::OFF)
            .optimizer(mosek::Optimizertype::INTPNT);
        assert_eq!(
            opts.double_params,
            vec![(Dparam::MIO_TOL_REL_GAP, 0.1), (Dparam::MIO_TOL_REL_GAP, 0.01)]
        );
        assert_eq!(opts.int_params.len(), 2);
    }
}
