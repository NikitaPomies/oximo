use crate::arena::{ExprArenaCell, ExprId, ExprNode, ParamId, VarId};
use crate::classify::{ExprClass, classify_access};

/// Lightweight handle to a node in an [`ExprArena`].
///
/// Carries a borrow of the arena cell so operator overloads can push new nodes
/// during arithmetic. `Expr` is `Copy`, so users freely reuse a variable
/// handle in many constraints.
#[derive(Copy, Clone)]
pub struct Expr<'a> {
    pub id: ExprId,
    pub arena: &'a ExprArenaCell,
}

impl std::fmt::Debug for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Expr").field("id", &self.id).finish()
    }
}

impl<'a> Expr<'a> {
    #[inline]
    pub fn new(id: ExprId, arena: &'a ExprArenaCell) -> Self {
        Self { id, arena }
    }

    pub fn constant(arena: &'a ExprArenaCell, v: f64) -> Self {
        let id = arena.with_mut(|arena| arena.constant(v));
        Self::new(id, arena)
    }

    pub fn from_var(arena: &'a ExprArenaCell, v: VarId) -> Self {
        let id = arena.with_mut(|arena| arena.var(v));
        Self::new(id, arena)
    }

    #[inline]
    pub(crate) fn assert_same_arena(self, other: Self) {
        assert!(std::ptr::eq(self.arena, other.arena), "expressions belong to different arenas");
    }

    /// If this handle is a bare variable, return its [`VarId`].
    /// `None` for compound expressions (sums, products, constants, ...).
    pub fn var_id(self) -> Option<VarId> {
        self.arena.with_ref(|arena| match arena.get(self.id) {
            ExprNode::Var(id) => Some(*id),
            _ => None,
        })
    }

    /// If this handle is a bare parameter, return its [`ParamId`].
    /// `None` for compound expressions.
    pub fn param_id(self) -> Option<ParamId> {
        self.arena.with_ref(|arena| match arena.get(self.id) {
            ExprNode::Param(id) => Some(*id),
            _ => None,
        })
    }

    /// Re-bind the parameter this handle references to `value`. Takes effect on
    /// the next extraction/evaluation, which read the value straight from the
    /// arena.
    ///
    /// # Panics
    /// Panics if this handle is not a bare parameter (see [`Self::param_id`]).
    pub fn set_param_value(self, value: f64) {
        let id = self.param_id().expect("set_param_value expects a bare parameter handle");
        self.arena.borrow_mut().set_param_value(id, value);
    }

    pub fn pow(self, exponent: Self) -> Self {
        self.assert_same_arena(exponent);
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Pow(self.id, exponent.id)));
        Self::new(id, self.arena)
    }

    pub fn powi(self, n: i32) -> Self {
        let id = self.arena.with_mut(|arena| {
            let exp_id = arena.constant(f64::from(n));
            arena.push(ExprNode::Pow(self.id, exp_id))
        });
        Self::new(id, self.arena)
    }

    pub fn powf(self, n: f64) -> Self {
        let id = self.arena.with_mut(|arena| {
            let exp_id = arena.constant(n);
            arena.push(ExprNode::Pow(self.id, exp_id))
        });
        Self::new(id, self.arena)
    }

    pub fn sin(self) -> Self {
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Sin(self.id)));
        Self::new(id, self.arena)
    }

    pub fn cos(self) -> Self {
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Cos(self.id)));
        Self::new(id, self.arena)
    }

    pub fn exp(self) -> Self {
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Exp(self.id)));
        Self::new(id, self.arena)
    }

    pub fn log(self) -> Self {
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Log(self.id)));
        Self::new(id, self.arena)
    }

    pub fn abs(self) -> Self {
        let id = self.arena.with_mut(|arena| arena.push(ExprNode::Abs(self.id)));
        Self::new(id, self.arena)
    }

    #[doc(hidden)]
    pub fn __class(self) -> ExprClass {
        self.arena.with_ref(|arena| classify_access(arena, self.id))
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;
    use crate::arena::{ExprArena, ExprArenaCell};

    #[test]
    fn set_param_value_rebinds_through_handle() {
        let arena = ExprArenaCell::new(ExprArena::new());
        let pid = arena.borrow_mut().new_param(0.05);
        let node = arena.borrow_mut().param(pid);
        let p = Expr::new(node, &arena);

        p.set_param_value(0.2);
        assert!((arena.borrow().param_value(pid) - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "bare parameter handle")]
    fn set_param_value_panics_on_non_param() {
        let arena = ExprArenaCell::new(ExprArena::new());
        let c = Expr::constant(&arena, 1.0);
        c.set_param_value(3.0);
    }

    #[test]
    #[should_panic(expected = "different arenas")]
    fn combining_different_arenas_is_rejected() {
        let left_arena = ExprArenaCell::new(ExprArena::new());
        let right_arena = ExprArenaCell::new(ExprArena::new());
        let left = Expr::constant(&left_arena, 1.0);
        let right = Expr::constant(&right_arena, 2.0);
        let _ = left + right;
    }
}
