use oximo_core::prelude::*;

#[test]
fn sos_macros_register_and_promote_kind() {
    let m = Model::new("sos");
    variable!(m, x);
    variable!(m, y);
    variable!(m, z);
    objective!(m, Min, x + y + z);
    sos_constraint!(m, first, SOS1, [(x, 1.0), (y, 2.0)]);
    sos_constraint!(m, curve, SOS2, [(x, 1.0), (y, 2.0), (z, 3.0)]);
    sos_constraint!(m, inferred, SOS1, [x, y, z]);
    assert_eq!(m.kind(), ModelKind::MILP);
    assert_eq!(m.num_sos_constraints(), 3);
    assert_eq!(m.sos_constraint_id("curve"), Some(SosConstraintId(1)));
    assert_eq!(m.sos_constraints()[1].sos_type, SosType::Sos2);
    assert_eq!(
        m.sos_constraints()[2].members.iter().map(|m| m.weight).collect::<Vec<_>>(),
        [1.0, 2.0, 3.0]
    );
}

#[test]
fn sos_family_and_auto_name_work() {
    let m = Model::new("sos_family");
    variable!(m, x[i in 0..2]);
    sos_constraint!(m, family[i in 0..2], SOS1, [(x[i], 1.0)]);
    sos_constraint!(m, SOS2, [(x[0], 1.0), (x[1], 2.0)]);
    assert_eq!(m.sos_constraints()[0].name, "family[0]");
    assert!(m.sos_constraints()[2].name.starts_with("_sos"));
}

#[test]
fn sos_indexed_family_infers_weights() {
    let m = Model::new("sos_indexed_family");
    variable!(m, x[i in 0..2]);
    variable!(m, y[i in 0..2]);
    variable!(m, z[i in 0..2]);
    sos_constraint!(m, choice[i in 0..2], SOS1, [x[i], y[i], z[i]]);

    assert_eq!(m.num_sos_constraints(), 2);
    assert_eq!(m.sos_constraints()[0].name, "choice[0]");
    assert_eq!(m.sos_constraints()[1].name, "choice[1]");
    for constraint in m.sos_constraints().iter() {
        assert_eq!(
            constraint.members.iter().map(|m| m.weight).collect::<Vec<_>>(),
            [1.0, 2.0, 3.0]
        );
    }
}

#[test]
fn sos_auto_weight_method_accepts_dynamic_members() {
    let m = Model::new("sos_dynamic");
    variable!(m, x);
    variable!(m, y);
    variable!(m, z);
    let members = vec![x, y, z];
    m.add_sos_constraint_auto_weights("dynamic", SosType::Sos2, members);

    assert_eq!(
        m.sos_constraints()[0].members.iter().map(|m| m.weight).collect::<Vec<_>>(),
        [1.0, 2.0, 3.0]
    );
}

#[test]
fn sos_registry_and_display_are_complete() {
    let m = Model::new("sos_display");
    variable!(m, x);
    variable!(m, y);
    let id = m.add_sos_constraint("choice", SosType::Sos2, [(x, 10.0), (y, 20.0)]);

    assert_eq!(id.index(), 0);
    assert_eq!(SosType::Sos1.label(), "SOS1");
    assert_eq!(SosType::Sos2.label(), "SOS2");
    assert!(m.has_sos_constraints());
    assert_eq!(m.sos_constraint_id("choice"), Some(id));
    assert_eq!(m.sos_constraint_id("missing"), None);
    assert_eq!(m.display_sos(id).to_string(), "choice: SOS2 [x: 10, y: 20]");
    assert!(m.to_string().contains("choice: SOS2 [x: 10, y: 20]"));
    assert_eq!(m.constraints().special_ordered_sets()[0].name, "choice");
}

#[test]
#[should_panic(expected = "has no members")]
fn sos_rejects_empty_members() {
    let m = Model::new("empty");
    m.add_sos_constraint("empty", SosType::Sos1, []);
}

#[test]
#[should_panic(expected = "duplicate variable")]
fn sos_rejects_duplicate_variables() {
    let m = Model::new("duplicate_variable");
    variable!(m, x);
    m.add_sos_constraint("duplicate", SosType::Sos1, [(x, 1.0), (x, 2.0)]);
}

#[test]
#[should_panic(expected = "non-finite weight")]
fn sos_rejects_nonfinite_weights() {
    let m = Model::new("nonfinite");
    variable!(m, x);
    m.add_sos_constraint("nonfinite", SosType::Sos1, [(x, f64::NAN)]);
}

#[test]
#[should_panic(expected = "must be bare variables")]
fn sos_rejects_compound_members() {
    let m = Model::new("compound");
    variable!(m, x);
    m.add_sos_constraint("compound", SosType::Sos1, [(x + 1.0, 1.0)]);
}

#[test]
#[should_panic(expected = "belongs to another model")]
fn sos_rejects_foreign_members() {
    let m = Model::new("owner");
    let other = Model::new("other");
    variable!(other, x);
    m.add_sos_constraint("foreign", SosType::Sos1, [(x, 1.0)]);
}

#[test]
#[should_panic(expected = "already registered")]
fn sos_rejects_duplicate_names() {
    let m = Model::new("duplicate_name");
    variable!(m, x);
    m.add_sos_constraint("same", SosType::Sos1, [(x, 1.0)]);
    m.add_sos_constraint("same", SosType::Sos1, [(x, 2.0)]);
}

#[test]
#[should_panic(expected = "duplicate weights")]
fn sos_rejects_duplicate_weights() {
    let m = Model::new("bad");
    variable!(m, x);
    variable!(m, y);
    m.add_sos_constraint("bad", SosType::Sos2, [(x, 1.0), (y, 1.0)]);
}

#[test]
#[should_panic(expected = "duplicate weights")]
fn sos_rejects_signed_zero_duplicate_weights() {
    let m = Model::new("signed_zero_weights");
    variable!(m, x);
    variable!(m, y);
    m.add_sos_constraint("signed_zero", SosType::Sos1, [(x, 0.0), (y, -0.0)]);
}
