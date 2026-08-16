use std::io::Cursor;

use oximo_core::prelude::*;
use oximo_io::{IoError, read_lp, read_lp_file, to_lp_string};

#[test]
fn reads_milp_sections_bounds_and_comments() {
    let text = r"
\* a comment *\
Maximize
 obj: 2 x + 3 y + 4
Subject To
 c1: x + y <= 10
 c2: x - 2 y >= -3
 c3: x + z = 2
Bounds
 -infinity <= x <= 10
 y >= 2
  1 <= z
 z = 1
General
 y
Binaries
 z
End
";
    let model = read_lp(Cursor::new(text)).expect("LP should parse");
    assert_eq!(model.num_variables(), 3);
    assert_eq!(model.num_constraints(), 3);
    assert_eq!(model.variables()[0].name, "x");
    assert!(model.variables()[0].lb.is_infinite() && model.variables()[0].lb.is_sign_negative());
    assert!((model.variables()[0].ub - 10.0).abs() < f64::EPSILON);
    assert!(matches!(model.variables()[1].domain, Domain::Integer));
    assert!(matches!(model.variables()[2].domain, Domain::Binary));
    assert!((model.variables()[2].lb - 1.0).abs() < f64::EPSILON);
    assert!((model.variables()[2].ub - 1.0).abs() < f64::EPSILON);
    assert_eq!(model.try_objective().expect("objective").sense, ObjectiveSense::Maximize);
}

#[test]
fn reads_quadratic_objective_and_constraint() {
    let text = r"
Minimize
 obj: x + [ x^2 + 4 x*y ] / 2
Subject To
 q: x^2 + y^2 <= 9
End
";
    let model = read_lp(text.as_bytes()).expect("quadratic LP should parse");
    assert_eq!(model.num_variables(), 2);
    assert_eq!(model.num_constraints(), 1);
    assert!(matches!(model.kind(), ModelKind::QCP | ModelKind::QP));
}

#[test]
fn accepts_moi_style_aliases_and_comparisons() {
    let text = "minimum obj: x\nsubject to\n c1: x < 2\n c2: x > -3\n c3: x == 1\n c4: x =< 4\n c5: x => -4\ninteger\n x\nEnd\n";
    let model = read_lp(text.as_bytes()).expect("aliases should parse");
    assert_eq!(model.num_constraints(), 5);
    assert!(matches!(model.variables()[0].domain, Domain::Integer));
}

#[test]
fn constraint_rhs_can_continue_on_next_line() {
    let text = "Minimize\n obj: x\nSubject To\n c: x <=\n 5\nEnd\n";
    let model = read_lp(text.as_bytes()).expect("continued RHS should parse");
    assert_eq!(model.num_constraints(), 1);
}

#[test]
fn unnamed_constraint_names_skip_explicit_names() {
    let text = "Minimize\n obj: x\nSubject To\n x <= 5\n c0: x >= 0\n x = 2\nEnd\n";
    let model = read_lp(text.as_bytes()).expect("constraint names should be unique");
    let constraints = model.constraints();
    let names: Vec<_> = constraints.algebraic().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["c1", "c0", "c2"]);
}

#[test]
fn negative_upper_bound_implies_free_lower_bound() {
    let model = read_lp("Minimize\n obj: x\nBounds\n x <= -1\nEnd\n".as_bytes())
        .expect("bound should parse");
    assert!(model.variables()[0].lb.is_infinite() && model.variables()[0].lb.is_sign_negative());
    assert!((model.variables()[0].ub + 1.0).abs() < f64::EPSILON);
}

#[test]
fn writer_output_round_trips() {
    let m = Model::new("roundtrip");
    variable!(m, 0.0 <= x <= 5.0);
    variable!(m, y, Integer);
    constraint!(m, c, x + y >= 2.0);
    objective!(m, Min, x + 2.0 * y);
    let text = to_lp_string(&m).expect("write LP");
    let got = read_lp(text.as_bytes()).expect("read written LP");
    assert_eq!(got.num_variables(), 2);
    assert_eq!(got.num_constraints(), 1);
    assert!(matches!(got.variables()[1].domain, Domain::Integer));
}

#[test]
fn writer_preserves_objective_constant() {
    let m = Model::new("constant");
    variable!(m, x);
    objective!(m, Min, x + 4.0);
    let text = to_lp_string(&m).expect("write LP");
    assert!(text.contains("+ 4"), "{text}");
    let got = read_lp(text.as_bytes()).expect("read written LP");
    assert!(got.try_objective().is_ok());
}

#[test]
fn writer_formats_negative_objective_constant() {
    let m = Model::new("negative_constant");
    variable!(m, x);
    objective!(m, Min, x - 4.0);
    let text = to_lp_string(&m).expect("write LP");
    assert!(text.contains("obj: x - 4"), "{text}");
}

#[test]
fn file_reader_uses_stem_as_model_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("example.lp");
    std::fs::write(&path, "Minimize\n obj: x\nEnd\n").expect("write fixture");
    let model = read_lp_file(&path).expect("read fixture");
    assert_eq!(model.name, "example");
}

#[test]
fn unsupported_sections_are_reported() {
    let err = read_lp("Minimize\n obj: x\nSOS\n s1: S1:: x 1\nEnd\n".as_bytes()).unwrap_err();
    assert!(matches!(err, IoError::UnsupportedLp { .. }));
}

#[test]
fn inline_sos_and_indicators_are_explicitly_unsupported() {
    for row in [" c: S1:: x:1 y:2", " c: z = 1 -> x <= 2"] {
        let text = format!("Minimize\n obj: x\nSubject To\n{row}\nEnd\n");
        let err = read_lp(text.as_bytes()).unwrap_err();
        assert!(matches!(err, IoError::UnsupportedLp { .. }), "{err:?}");
    }
}

#[test]
fn content_after_end_is_rejected() {
    let err = read_lp("Minimize\n obj: x\nEnd\nBounds\n x >= 0\n".as_bytes()).unwrap_err();
    assert!(matches!(err, IoError::InvalidLp { .. }));
}

#[test]
fn malformed_input_has_lp_diagnostic() {
    let err =
        read_lp("Minimize\n obj: x\nSubject To\n bad: x <= nope\nEnd\n".as_bytes()).unwrap_err();
    assert!(matches!(err, IoError::InvalidLp { .. }));
}

#[test]
fn higher_degree_expression_is_invalid_lp_syntax() {
    let err = read_lp("Minimize\n obj: x^3\nEnd\n".as_bytes()).unwrap_err();
    assert!(matches!(err, IoError::InvalidLp { .. }));
}
