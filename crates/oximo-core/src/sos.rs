use oximo_expr::VarId;
use rustc_hash::{FxBuildHasher, FxHashSet};
use smol_str::SmolStr;

use crate::model::Model;
use crate::reformulation::{
    ReformulatedModel, ReformulationError, SosReformulationArtifacts, SosReformulationOptions,
};

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

impl std::fmt::Display for SosType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
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

/// A model-bound handle to an SOS constraint.
///
/// Single-constraint forms of [`crate::sos_constraint!`] and the programmatic
/// SOS registration methods return this handle.
#[derive(Copy, Clone)]
pub struct SosConstraintHandle<'a> {
    pub(crate) model: &'a Model,
    pub(crate) id: SosConstraintId,
}

impl std::fmt::Debug for SosConstraintHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SosConstraintHandle").field("id", &self.id).finish()
    }
}

impl SosConstraintHandle<'_> {
    #[must_use]
    pub const fn id(self) -> SosConstraintId {
        self.id
    }

    #[must_use]
    pub fn index(self) -> usize {
        self.id.index()
    }

    /// Produce an independent model in which this SOS constraint is replaced
    /// by mixed-integer algebraic constraints.
    ///
    /// # Errors
    ///
    /// Returns [`ReformulationError`] when a fallback Big-M is invalid or a
    /// member lacks a finite bound and no fallback was supplied.
    pub fn to_reformulated_model(
        self,
        options: SosReformulationOptions,
    ) -> Result<ReformulatedModel, ReformulationError> {
        self.model.to_reformulated_sos_constraint_model(self.id, options)
    }

    /// Replace this SOS constraint on its source model without cloning the
    /// model. Returns `Ok(None)` if it was already reformulated.
    ///
    /// # Errors
    ///
    /// Returns [`ReformulationError`] before modifying the model when the
    /// fallback Big-M or a required member bound is invalid.
    pub fn reformulate(
        self,
        options: SosReformulationOptions,
    ) -> Result<Option<SosReformulationArtifacts>, ReformulationError> {
        self.model.reformulate_sos_constraint(self.id, options)
    }
}

impl From<SosConstraintHandle<'_>> for SosConstraintId {
    fn from(value: SosConstraintHandle<'_>) -> Self {
        value.id
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
    let mut vars = FxHashSet::with_capacity_and_hasher(members.len(), FxBuildHasher);
    let mut weights = FxHashSet::with_capacity_and_hasher(members.len(), FxBuildHasher);
    for member in members {
        assert!(member.weight.is_finite(), "SOS constraint {name:?} has a non-finite weight");
        assert!(
            vars.insert(member.variable),
            "SOS constraint {name:?} contains a duplicate variable"
        );
        let key = match member.weight.to_bits() {
            0 | 0x8000_0000_0000_0000 => 0,
            bits => bits,
        };
        assert!(weights.insert(key), "SOS constraint {name:?} contains duplicate weights");
    }
}
