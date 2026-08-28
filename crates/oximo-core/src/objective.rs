use std::fmt;

use oximo_expr::ExprId;

/// Whether the model minimizes or maximizes its objective.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObjectiveSense {
    Minimize,
    Maximize,
}

impl fmt::Display for ObjectiveSense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
        })
    }
}

/// The model's objective: an expression to optimize and the direction.
#[derive(Clone, Debug)]
pub struct Objective {
    /// Root node of the objective expression in the model's [`oximo_expr::ExprArena`].
    pub expr: ExprId,
    pub sense: ObjectiveSense,
}

#[cfg(test)]
mod tests {
    use super::ObjectiveSense;

    #[test]
    fn display_uses_user_facing_ascii_labels() {
        let labels = [ObjectiveSense::Minimize.to_string(), ObjectiveSense::Maximize.to_string()];
        assert_eq!(labels, ["minimize", "maximize"]);
        assert!(labels.iter().all(|label| label.is_ascii()));
    }
}
