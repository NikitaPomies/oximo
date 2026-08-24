use std::collections::HashSet;

use oximo_expr::VarId;
use smol_str::SmolStr;

/// The two standard special ordered set types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SosType {
    /// At most one member may be nonzero.
    Sos1,
    /// At most two adjacent (by weight) members may be nonzero.
    Sos2,
}

impl SosType {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sos1 => "SOS1",
            Self::Sos2 => "SOS2",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SosMember {
    pub variable: VarId,
    pub weight: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SosConstraintId(pub u32);

impl SosConstraintId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// An explicit SOS1 or SOS2 constraint.
#[derive(Clone, Debug)]
pub struct SosConstraint {
    pub name: SmolStr,
    pub sos_type: SosType,
    pub members: Vec<SosMember>,
    pub active: bool,
}

/// Validate an SOS member list before it is stored in a model.
pub(crate) fn validate_members(name: &str, members: &[SosMember]) {
    assert!(!members.is_empty(), "SOS constraint {name:?} has no members");
    let mut vars = HashSet::with_capacity(members.len());
    for member in members {
        assert!(member.weight.is_finite(), "SOS constraint {name:?} has a non-finite weight");
        assert!(
            vars.insert(member.variable),
            "SOS constraint {name:?} contains a duplicate variable"
        );
    }
    let mut weights = HashSet::with_capacity(members.len());
    for member in members {
        let key = match member.weight.to_bits() {
            0 | 0x8000_0000_0000_0000 => 0,
            bits => bits,
        };
        assert!(weights.insert(key), "SOS constraint {name:?} contains duplicate weights");
    }
}
