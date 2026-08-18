use std::io::{self, Read};

use oximo_core::prelude::*;
use oximo_expr::extract_quadratic;
use oximo_io::{
    IoError, MpsQuadraticFormat, MpsReadOptions, MpsWriteOptions, read_mps, read_mps_file,
    read_mps_with, to_mps_string, to_mps_string_with,
};

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-12
}

fn quadratic_terms(model: &Model, objective: bool) -> oximo_expr::QuadraticTerms {
    let arena = model.arena();
    let expr = if objective {
        model.try_objective().expect("objective").expr
    } else {
        model.constraints().algebraic()[0].lhs
    };
    extract_quadratic(&arena, expr).expect("quadratic expression")
}

#[test]
fn reads_linear_milp_ranges_defaults_and_duplicate_coefficients() {
    let text = r"
* comment
NAME mixed
OBJSENSE MAX
ROWS
 N obj
 L cap
 G floor
 N free_row
COLUMNS
 marker 'MARKER' 'INTORG'
 x obj 1D0 cap 2
 x obj 2
 marker 'MARKER' 'INTEND'
 y floor 1 free_row -1
RHS
 rhs cap 10 floor 3
 rhs obj -5
RANGES
 rng cap 4
BOUNDS
 FR bnd y
ENDATA
";
    let model = read_mps(text.as_bytes()).expect("MPS should parse");
    assert_eq!(model.name, "mixed");
    assert_eq!(model.num_variables(), 2);
    assert_eq!(model.num_constraints(), 3);
    assert_eq!(model.try_objective().expect("objective").sense, ObjectiveSense::Maximize);
    assert!(matches!(model.variables()[0].domain, Domain::Integer));
    assert!(close(model.variables()[0].lb, 0.0));
    assert!(close(model.variables()[0].ub, 1.0));
    assert!(model.variables()[1].lb.is_infinite() && model.variables()[1].lb.is_sign_negative());

    let objective = quadratic_terms(&model, true);
    assert_eq!(objective.linear, vec![(model.variables()[0].id, 3.0)]);
    assert!(close(objective.constant, 5.0));
    let constraints = model.constraints();
    assert!(close(constraints.algebraic()[0].lower, 6.0));
    assert!(close(constraints.algebraic()[0].upper, 10.0));
    assert!(constraints.algebraic()[2].lower.is_infinite());
    assert!(constraints.algebraic()[2].upper.is_infinite());
}

#[test]
fn upstream_stacked_data_fixture_with_tabs_and_extra_row_fields() {
    let text = "NAME stacked\nROWS\n N OBJ metadata\n L limit metadata\nCOLUMNS\n x\tOBJ\t1\tlimit\t2\n y\tOBJ\t-1\tlimit\t3\nRHS\n rhs\tOBJ\t4\tlimit\t7\nENDATA\n";
    let model = read_mps(text.as_bytes()).expect("stacked upstream fixture");

    assert_eq!(model.num_variables(), 2);
    assert_eq!(model.num_constraints(), 1);
    let objective = quadratic_terms(&model, true);
    assert_eq!(objective.linear.len(), 2);
    assert!(close(objective.constant, -4.0));
    let constraints = model.constraints();
    let row = &constraints.algebraic()[0];
    assert!(row.lower.is_infinite() && row.lower.is_sign_negative());
    assert!(close(row.upper, 7.0));
}

#[test]
fn upstream_integer_default_bounds_reset_on_explicit_bound() {
    let text = "NAME integer\nROWS\n N OBJ\nCOLUMNS\n marker 'MARKER' 'INTORG'\n x OBJ 1\n marker 'MARKER' 'INTEND'\n y OBJ 1\nBOUNDS\n LO BND x 2\nENDATA\n";
    let model = read_mps(text.as_bytes()).expect("integer default bounds fixture");
    let x = &model.variables()[0];
    assert!(matches!(x.domain, Domain::Integer));
    assert!(close(x.lb, 2.0));
    assert!(x.ub.is_infinite() && x.ub.is_sign_positive());
    assert!(matches!(model.variables()[1].domain, Domain::Real));
}

#[test]
fn legacy_sense_is_fallback_and_objsense_takes_precedence() {
    let legacy = "* sense: maximize\nNAME old\nROWS\n N obj\nCOLUMNS\n x obj 1\nENDATA\n";
    let model = read_mps(legacy.as_bytes()).expect("legacy sense");
    assert_eq!(model.try_objective().expect("objective").sense, ObjectiveSense::Maximize);

    let standard =
        "* sense: maximize\nNAME current\nOBJSENSE MIN\nROWS\n N obj\nCOLUMNS\n x obj 1\nENDATA\n";
    let model = read_mps(standard.as_bytes()).expect("standard sense");
    assert_eq!(model.try_objective().expect("objective").sense, ObjectiveSense::Minimize);
}

#[test]
fn reads_bound_types_and_semi_domains() {
    let text = r"
NAME domains
ROWS
 N obj
COLUMNS
 a obj 1
 b obj 1
 c obj 1
 d obj 1
 e obj 1
BOUNDS
 BV bnd a
 LI bnd b -2
 UI bnd b 7
 LO bnd c 2
 SC bnd c 10
 LO bnd d 1
 SI bnd d 5
 FX bnd e 4
ENDATA
";
    let model = read_mps(text.as_bytes()).expect("domains");
    let vars = model.variables();
    assert!(matches!(vars[0].domain, Domain::Binary));
    assert!(matches!(vars[1].domain, Domain::Integer));
    assert_eq!((vars[1].lb, vars[1].ub), (-2.0, 7.0));
    assert!(matches!(vars[2].domain, Domain::SemiContinuous { threshold: 2.0 }));
    assert!(close(vars[2].ub, 10.0));
    assert!(matches!(vars[3].domain, Domain::SemiInteger { threshold: 1.0 }));
    assert_eq!((vars[4].lb, vars[4].ub), (4.0, 4.0));
}

#[test]
fn writer_round_trips_max_binary_semi_range_and_unused_variables() {
    let model = Model::new("roundtrip");
    variable!(model, x <= 8.0);
    variable!(model, b, Binary);
    variable!(model, s <= 10.0, SemiCont(2.0));
    variable!(model, unused);
    let _ = unused;
    model.__add_constraint_interval("range", x + b, 1.0, 6.0);
    objective!(model, Max, 2.0 * x + b + s + 4.0);

    let text = to_mps_string(&model).expect("write MPS");
    assert!(text.contains("OBJSENSE\n MAX"), "{text}");
    assert!(text.lines().any(|line| line.contains("BV BND") && line.contains('b')), "{text}");
    assert!(text.lines().any(|line| line.contains("unused") && line.contains("OBJ")), "{text}");

    let imported = read_mps(text.as_bytes()).expect("read writer output");
    assert_eq!(imported.num_variables(), 4);
    assert_eq!(imported.variables()[3].name, "unused");
    assert!(matches!(imported.variables()[1].domain, Domain::Binary));
    assert!(matches!(imported.variables()[2].domain, Domain::SemiContinuous { threshold: 2.0 }));
    assert_eq!(imported.try_objective().expect("objective").sense, ObjectiveSense::Maximize);
    let constraints = imported.constraints();
    assert_eq!((constraints.algebraic()[0].lower, constraints.algebraic()[0].upper), (1.0, 6.0));
    assert!(close(quadratic_terms(&imported, true).constant, 4.0));
}

#[test]
fn writer_round_trips_quadratic_objective_and_constraints_for_mps_dialects() {
    let model = Model::new("quadratic roundtrip");
    let x = model.__var("x").bounds(-2.0, 4.0).build();
    let y = model.__var("y").bounds(-3.0, 5.0).build();
    model.__add_constraint("q", (x * x + 2.0 * x * y + y * y).le(10.0));
    model.__minimize(3.0 * x * x + 4.0 * x * y + y * y + x);

    let expected_objective = quadratic_terms(&model, true);
    let expected_constraint = quadratic_terms(&model, false);
    for format in [MpsQuadraticFormat::Gurobi, MpsQuadraticFormat::Cplex, MpsQuadraticFormat::Mosek]
    {
        let options = MpsWriteOptions { quadratic_format: format };
        let text = to_mps_string_with(&model, &options).expect("write quadratic MPS");
        let expected_header = match format {
            MpsQuadraticFormat::Mosek => "QSECTION",
            MpsQuadraticFormat::Gurobi | MpsQuadraticFormat::Cplex => "QUADOBJ",
        };
        assert!(text.contains(expected_header), "{text}");
        if format == MpsQuadraticFormat::Gurobi {
            assert!(
                text.contains("y          x"),
                "Gurobi QUADOBJ must use lower triangle:\n{text}"
            );
            assert!(text.matches("x          y").count() >= 1);
            assert!(text.matches("y          x").count() >= 1);
        } else if format == MpsQuadraticFormat::Cplex {
            assert!(
                text.contains("x          y"),
                "CPLEX QUADOBJ must use upper triangle:\n{text}"
            );
            assert!(text.matches("x          y").count() >= 1);
            assert!(text.matches("y          x").count() >= 1);
        } else {
            assert!(
                text.contains("y          x"),
                "MOSEK QSECTION must use lower triangle:\n{text}"
            );
        }
        let imported = read_mps_with(text.as_bytes(), &MpsReadOptions { quadratic_format: format })
            .expect("read quadratic MPS");
        let actual_objective = quadratic_terms(&imported, true);
        let actual_constraint = quadratic_terms(&imported, false);
        assert_eq!(actual_objective.hessian.len(), expected_objective.hessian.len());
        assert_eq!(actual_constraint.hessian.len(), expected_constraint.hessian.len());
        for (actual, expected) in actual_objective.hessian.iter().zip(&expected_objective.hessian) {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert!(
                close(actual.2, expected.2),
                "{format:?} objective: actual={actual:?} expected={expected:?}"
            );
        }
        for (actual, expected) in actual_constraint.hessian.iter().zip(&expected_constraint.hessian)
        {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
            assert!(
                close(actual.2, expected.2),
                "{format:?} constraint: actual={actual:?} expected={expected:?}"
            );
        }
    }
}

#[test]
fn writer_sanitizes_and_uniquifies_mps_names() {
    let model = Model::new("name roundtrip");
    let spaced = model.__var("x one").build();
    let tabbed = model.__var("x\tone").build();
    let underscored = model.__var("x_one").build();
    let unnamed = model.__var("").build();
    model.__add_constraint("OBJ", spaced.le(1.0));
    model.__add_constraint("limit one", tabbed.ge(2.0));
    model.__add_constraint("limit_one", underscored.eq(3.0));
    model.__add_constraint("", unnamed.le(4.0));
    model.__minimize(spaced + tabbed + underscored + unnamed);

    let text = to_mps_string(&model).expect("write MPS");
    let imported = read_mps(text.as_bytes()).expect("read sanitized MPS");

    let variable_names: Vec<_> =
        imported.variables().iter().map(|variable| variable.name.to_string()).collect();
    assert_eq!(variable_names, ["x_one", "x_one_1", "x_one_2", "C4"]);
    let constraint_names: Vec<_> = imported
        .constraints()
        .algebraic()
        .iter()
        .map(|constraint| constraint.name.to_string())
        .collect();
    assert_eq!(constraint_names, ["OBJ_1", "limit_one", "limit_one_1", "R4"]);
}

#[test]
fn reads_quadratic_objective_matrix_once() {
    let text = r"
NAME qp
ROWS
 N obj
COLUMNS
 x obj 1
 y obj 1
QMATRIX
 x x 10
 x y 2
 y x 2
 y y 2
ENDATA
";
    let model = read_mps(text.as_bytes()).expect("QMATRIX");
    let q = quadratic_terms(&model, true);
    assert_eq!(q.hessian.len(), 3);
    assert!(q.hessian.iter().any(|(_, _, value)| close(*value, 10.0)));
    assert!(q.hessian.iter().any(|(row, col, value)| row != col && close(*value, 2.0)));
    assert!(q.hessian.iter().any(|(row, col, value)| row == col && close(*value, 2.0)));
}

#[test]
fn reads_quadobj_triangular_objective() {
    let text =
        "NAME qp\nROWS\n N obj\nCOLUMNS\n x obj 0\n y obj 0\nQUADOBJ\n x x 8\n x y 3\nENDATA\n";
    let model = read_mps(text.as_bytes()).expect("QUADOBJ");
    let q = quadratic_terms(&model, true);
    assert!(q.hessian.iter().any(|(row, col, value)| row == col && close(*value, 8.0)));
    assert!(q.hessian.iter().any(|(row, col, value)| row != col && close(*value, 3.0)));
}

#[test]
fn quadratic_constraint_scaling_is_configurable() {
    let text = r"
NAME qcp
ROWS
 N obj
 L q
COLUMNS
 x q 1
 y q 1
RHS
 rhs q 1
QCMATRIX q
 x x 10
 x y 2
 y x 2
 y y 2
ENDATA
";
    let gurobi = read_mps(text.as_bytes()).expect("Gurobi scaling");
    let gurobi_q = quadratic_terms(&gurobi, false);
    assert!(gurobi_q.hessian.iter().any(|(row, col, value)| row == col && close(*value, 20.0)));
    assert!(gurobi_q.hessian.iter().any(|(row, col, value)| row != col && close(*value, 4.0)));

    let options = MpsReadOptions { quadratic_format: MpsQuadraticFormat::Cplex };
    let cplex = read_mps_with(text.as_bytes(), &options).expect("CPLEX scaling");
    let cplex_q = quadratic_terms(&cplex, false);
    assert!(cplex_q.hessian.iter().any(|(row, col, value)| row == col && close(*value, 10.0)));
    assert!(cplex_q.hessian.iter().any(|(row, col, value)| row != col && close(*value, 2.0)));
}

#[test]
fn qsection_supports_objective_and_constraint_targets() {
    let declared = "NAME q\nROWS\n N cost\nCOLUMNS\n x cost 0\nQSECTION cost\n x x 6\nENDATA\n";
    let model = read_mps(declared.as_bytes()).expect("declared objective QSECTION");
    assert!(close(quadratic_terms(&model, true).hessian[0].2, 6.0));

    let obj_alias = "NAME q\nROWS\n N cost\nCOLUMNS\n x cost 0\nQSECTION OBJ\n x x 8\nENDATA\n";
    let model = read_mps(obj_alias.as_bytes()).expect("OBJ objective QSECTION");
    assert!(close(quadratic_terms(&model, true).hessian[0].2, 8.0));

    let constraint =
        "NAME q\nROWS\n N obj\n L row\nCOLUMNS\n x row 0\nQSECTION row\n x x 3\nENDATA\n";
    let model = read_mps(constraint.as_bytes()).expect("constraint QSECTION");
    assert!(close(quadratic_terms(&model, false).hessian[0].2, 6.0));
}

#[test]
fn qsection_obj_rejects_ambiguous_and_duplicate_objective_targets() {
    let ambiguous =
        "NAME q\nROWS\n N cost\n L OBJ\nCOLUMNS\n x cost 0 OBJ 0\nQSECTION OBJ\n x x 1\nENDATA\n";
    let error = read_mps(ambiguous.as_bytes()).expect_err("ambiguous OBJ target");
    assert!(matches!(
        error,
        IoError::InvalidMps { message, .. } if message.contains("ambiguous")
    ));

    let duplicate = "NAME q\nROWS\n N cost\nCOLUMNS\n x cost 0\nQSECTION cost\n x x 1\nQSECTION OBJ\n x x 1\nENDATA\n";
    let error = read_mps(duplicate.as_bytes()).expect_err("duplicate objective QSECTION");
    assert!(matches!(
        error,
        IoError::InvalidMps { message, .. } if message.contains("already supplied")
    ));
}

#[test]
fn feasibility_mps_gets_zero_minimization_objective() {
    let text = "NAME feasible\nROWS\n G row\nCOLUMNS\n x row 1\nENDATA\n";
    let model = read_mps(text.as_bytes()).expect("feasibility MPS");
    assert_eq!(model.try_objective().expect("objective").sense, ObjectiveSense::Minimize);
    let objective = quadratic_terms(&model, true);
    assert!(objective.linear.is_empty());
    assert!(close(objective.constant, 0.0));
}

#[test]
fn file_reader_uses_name_then_file_stem_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    let named = dir.path().join("fallback.mps");
    std::fs::write(&named, "NAME explicit\nROWS\n N obj\nCOLUMNS\n x obj 0\nENDATA\n")
        .expect("fixture");
    assert_eq!(read_mps_file(&named).expect("named").name, "explicit");

    std::fs::write(&named, "NAME\nROWS\n N obj\nCOLUMNS\n x obj 0\nENDATA\n").expect("fixture");
    assert_eq!(read_mps_file(&named).expect("fallback").name, "fallback");
}

#[test]
fn rejects_unsupported_sections_and_multiple_vectors() {
    for section in ["SOS", "INDICATORS"] {
        let text = format!("NAME bad\nROWS\n N obj\nCOLUMNS\n x obj 0\n{section}\nENDATA\n");
        assert!(matches!(read_mps(text.as_bytes()), Err(IoError::UnsupportedMps { .. })));
    }
    let vectors = "NAME bad\nROWS\n N obj\n L row\nCOLUMNS\n x row 1\nRHS\n first row 1\n second row 2\nENDATA\n";
    assert!(matches!(read_mps(vectors.as_bytes()), Err(IoError::UnsupportedMps { .. })));
}

#[test]
fn malformed_inputs_return_mps_diagnostics() {
    for text in [
        "NAME bad\nROWS\n N obj\nCOLUMNS\n x missing 1\nENDATA\n",
        "NAME bad\nROWS\n N obj\nCOLUMNS\n mark 'MARKER' 'INTORG'\n x obj 1\nENDATA\n",
        "NAME bad\nROWS\n N obj\nCOLUMNS\n x obj NaN\nENDATA\n",
        "NAME bad\nROWS\n N obj\nCOLUMNS\n x obj 1\n",
        "NAME bad\nROWS\n N obj\nCOLUMNS\n x obj 1\nENDATA\nBOUNDS\n",
    ] {
        assert!(matches!(read_mps(text.as_bytes()), Err(IoError::InvalidMps { .. })), "{text}");
    }
}

#[test]
fn stream_and_file_io_errors_are_preserved() {
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    assert!(matches!(read_mps(FailingReader), Err(IoError::Io(_))));
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(matches!(read_mps_file(dir.path().join("missing.mps")), Err(IoError::Io(_))));
}
