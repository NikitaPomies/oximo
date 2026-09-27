use oximo_solver::{HasUniversal, SolverError, UniversalOptions};
use russcip::{Model, ParamSetting, ProblemCreated};

/// SCIP's parameter presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScipSetting {
    Default,
    Fast,
    Aggressive,
    Off,
}

impl From<ScipSetting> for ParamSetting {
    fn from(value: ScipSetting) -> Self {
        match value {
            ScipSetting::Default => Self::Default,
            ScipSetting::Fast => Self::Fast,
            ScipSetting::Aggressive => Self::Aggressive,
            ScipSetting::Off => Self::Off,
        }
    }
}

/// Types supported by the safe upstream parameter setters.
#[derive(Clone, Debug, PartialEq)]
pub enum ScipOptionValue {
    Bool(bool),
    Int(i32),
    LongInt(i64),
    Real(f64),
    String(String),
}

impl From<bool> for ScipOptionValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i32> for ScipOptionValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}
impl From<i64> for ScipOptionValue {
    fn from(value: i64) -> Self {
        Self::LongInt(value)
    }
}
impl From<f64> for ScipOptionValue {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}
impl From<String> for ScipOptionValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&str> for ScipOptionValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

/// Options are reapplied from defaults on every solve. Named parameters override
/// presets and universal options, in insertion order. Invalid parameters fail.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScipOptions {
    pub universal: UniversalOptions,
    pub presolving: Option<ScipSetting>,
    pub separating: Option<ScipSetting>,
    pub heuristics: Option<ScipSetting>,
    /// Explicitly use SCIP's concurrent solver. Custom Rust plugins are not
    /// supported in this mode. `threads` sets parallel/maxnthreads, not this flag.
    pub concurrent: bool,
    pub(crate) parameters: Vec<(String, ScipOptionValue)>,
}

impl HasUniversal for ScipOptions {
    fn universal(&self) -> &UniversalOptions {
        &self.universal
    }
    fn universal_mut(&mut self) -> &mut UniversalOptions {
        &mut self.universal
    }
}

macro_rules! parameter {
    ($method:ident, $ty:ty, $setter:ident, $name:literal) => {
        #[must_use]
        pub fn $method(self, value: $ty) -> Self {
            self.$setter($name, value)
        }
    };
}

impl ScipOptions {
    #[must_use]
    pub fn param(mut self, name: impl Into<String>, value: impl Into<ScipOptionValue>) -> Self {
        self.parameters.push((name.into(), value.into()));
        self
    }
    #[must_use]
    pub fn bool_param(self, name: impl Into<String>, value: bool) -> Self {
        self.param(name, ScipOptionValue::Bool(value))
    }
    #[must_use]
    pub fn int_param(self, name: impl Into<String>, value: i32) -> Self {
        self.param(name, ScipOptionValue::Int(value))
    }
    #[must_use]
    pub fn longint_param(self, name: impl Into<String>, value: i64) -> Self {
        self.param(name, ScipOptionValue::LongInt(value))
    }
    #[must_use]
    pub fn real_param(self, name: impl Into<String>, value: f64) -> Self {
        self.param(name, ScipOptionValue::Real(value))
    }
    #[must_use]
    pub fn string_param(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.param(name, ScipOptionValue::String(value.into()))
    }
    #[must_use]
    pub fn presolving(mut self, value: ScipSetting) -> Self {
        self.presolving = Some(value);
        self
    }
    #[must_use]
    pub fn separating(mut self, value: ScipSetting) -> Self {
        self.separating = Some(value);
        self
    }
    #[must_use]
    pub fn heuristics(mut self, value: ScipSetting) -> Self {
        self.heuristics = Some(value);
        self
    }
    #[must_use]
    pub fn concurrent(mut self, value: bool) -> Self {
        self.concurrent = value;
        self
    }
    parameter!(mip_gap, f64, real_param, "limits/gap");
    parameter!(mip_gap_abs, f64, real_param, "limits/absgap");
    parameter!(node_limit, i64, longint_param, "limits/nodes");
    parameter!(total_node_limit, i64, longint_param, "limits/totalnodes");
    parameter!(stall_node_limit, i64, longint_param, "limits/stallnodes");
    parameter!(solution_limit, i32, int_param, "limits/solutions");
    parameter!(best_solution_limit, i32, int_param, "limits/bestsol");
    parameter!(max_solutions, i32, int_param, "limits/maxsol");
    parameter!(memory_limit, f64, real_param, "limits/memory");
    parameter!(feasibility_tol, f64, real_param, "numerics/feastol");
    parameter!(dual_feasibility_tol, f64, real_param, "numerics/dualfeastol");
    parameter!(random_seed, i32, int_param, "randomization/randomseedshift");
    parameter!(primal_limit, f64, real_param, "limits/primal");
    parameter!(dual_limit, f64, real_param, "limits/dual");

    pub(crate) fn apply(
        &self,
        mut model: Model<ProblemCreated>,
    ) -> Result<Model<ProblemCreated>, SolverError> {
        if let Some(v) = self.presolving {
            model = model.set_presolving(v.into());
        }
        if let Some(v) = self.separating {
            model = model.set_separating(v.into());
        }
        if let Some(v) = self.heuristics {
            model = model.set_heuristics(v.into());
        }
        model = model
            .set_int_param(
                "display/verblevel",
                if self.universal.verbose == Some(true) { 4 } else { 0 },
            )
            .map_err(crate::backend)?;
        if let Some(v) = self.universal.time_limit {
            model = model.set_real_param("limits/time", v.as_secs_f64()).map_err(crate::backend)?;
        }
        if let Some(v) = self.universal.threads {
            let v = i32::try_from(v).map_err(crate::backend)?;
            model = model.set_int_param("parallel/maxnthreads", v).map_err(crate::backend)?;
        }
        for (name, value) in &self.parameters {
            crate::check_name(name)?;
            model = match value {
                ScipOptionValue::Bool(v) => model.set_bool_param(name, *v),
                ScipOptionValue::Int(v) => model.set_int_param(name, *v),
                ScipOptionValue::LongInt(v) => model.set_longint_param(name, *v),
                ScipOptionValue::Real(v) => {
                    crate::finite(*v)?;
                    model.set_real_param(name, *v)
                }
                ScipOptionValue::String(v) => {
                    crate::check_name(v)?;
                    model.set_str_param(name, v)
                }
            }
            .map_err(|e| SolverError::Backend(format!("SCIP parameter {name}: {e:?}")))?;
        }
        Ok(model)
    }
}
