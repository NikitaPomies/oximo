#[path = "../../oximo-expr/benches/support/mod.rs"]
mod support;

use oximo_autodiff::tape::{CompiledBatch, CompiledExpr, Tape};
use oximo_expr::{ExprArena, ExprId, ExprNode, UnaryOp, VarId, evaluate};
use smallvec::smallvec;

#[global_allocator]
static ALLOC: support::CountingAllocator = support::CountingAllocator;

fn deep_neg_chain(arena: &mut ExprArena, depth: usize) -> ExprId {
    let mut root = arena.push(ExprNode::Var(VarId(0)));
    for _ in 0..depth {
        root = arena.push(ExprNode::Unary(UnaryOp::Neg, root));
    }
    root
}

/// `levels` doublings: `2^levels` paths, `levels` distinct nodes.
fn shared_doubling(arena: &mut ExprArena, levels: usize) -> ExprId {
    let mut root = arena.push(ExprNode::Var(VarId(0)));
    for _ in 0..levels {
        root = arena.push(ExprNode::Add(smallvec![root, root]));
    }
    root
}

/// One `sin(x0)` shared by `width` summands.
fn shared_sin_fanout(arena: &mut ExprArena, width: usize) -> ExprId {
    let x0 = arena.push(ExprNode::Var(VarId(0)));
    let sin = arena.push(ExprNode::Unary(UnaryOp::Sin, x0));
    let children: oximo_expr::Children = std::iter::repeat_n(sin, width).collect();
    arena.push(ExprNode::Add(children))
}

fn main() {
    println!("case,nanoseconds,allocations,requested_bytes,peak_live_bytes");
    let mut arena = ExprArena::new();
    let x: &[f64] = &[0.7];

    for depth in [256, 10_000] {
        let root = deep_neg_chain(&mut arena, depth);
        support::measure(&format!("tape_compile_deep/{depth}"), || Tape::compile(&arena, root));
        let tape = Tape::compile(&arena, root);
        let mut regs = vec![0.0; tape.n_regs()];
        support::measure(&format!("tape_value_reused_deep/{depth}"), || {
            tape.value(x, &[], &[], &mut regs)
        });
        let mut compiled = CompiledExpr::compile(&arena, root);
        support::measure(&format!("compiled_expr_deep/{depth}"), || compiled.eval(x, &[]));
        support::measure(&format!("evaluate_deep/{depth}"), || evaluate(&arena, root, &x).unwrap());
    }

    for levels in [14, 20] {
        let root = shared_doubling(&mut arena, levels);
        support::measure(&format!("tape_compile_shared/{levels}"), || Tape::compile(&arena, root));
        let tape = Tape::compile(&arena, root);
        assert!(
            tape.n_regs() <= levels + 2,
            "shared DAG should lower once per level, got {} regs for {levels} levels",
            tape.n_regs()
        );
        let mut regs = vec![0.0; tape.n_regs()];
        support::measure(&format!("tape_value_reused_shared/{levels}"), || {
            tape.value(x, &[], &[], &mut regs)
        });
        let mut compiled = CompiledExpr::compile(&arena, root);
        support::measure(&format!("compiled_expr_shared/{levels}"), || compiled.eval(x, &[]));
        support::measure(&format!("evaluate_shared/{levels}"), || {
            evaluate(&arena, root, &x).unwrap()
        });
    }

    for width in [32, 1024] {
        let root = shared_sin_fanout(&mut arena, width);
        support::measure(&format!("tape_compile_fanout/{width}"), || Tape::compile(&arena, root));
        let tape = Tape::compile(&arena, root);
        let mut regs = vec![0.0; tape.n_regs()];
        support::measure(&format!("tape_value_reused_fanout/{width}"), || {
            tape.value(x, &[], &[], &mut regs)
        });
    }

    // Batch: several outputs sharing one subexpression, one pass, one scratch.
    let x0 = arena.push(ExprNode::Var(VarId(0)));
    let sin = arena.push(ExprNode::Unary(UnaryOp::Sin, x0));
    let mut exprs = Vec::new();
    for k in 0..32 {
        let c = arena.push(ExprNode::Const(f64::from(k)));
        exprs.push(arena.push(ExprNode::Add(smallvec![sin, c])));
    }
    support::measure("batch_compile/32", || CompiledBatch::compile(&arena, &exprs));
    support::measure("separate_compile/32", || {
        exprs.iter().map(|&e| Tape::compile(&arena, e)).collect::<Vec<_>>()
    });
    let mut batch = CompiledBatch::compile(&arena, &exprs);
    let mut out = vec![0.0; exprs.len()];
    support::measure("batch_eval/32", || {
        batch.eval(x, &[], &mut out);
        std::hint::black_box(&out);
    });
    let tapes: Vec<Tape> = exprs.iter().map(|&e| Tape::compile(&arena, e)).collect();
    let mut regs: Vec<Vec<f64>> = tapes.iter().map(|t| vec![0.0; t.n_regs()]).collect();
    support::measure("tape_value_each_reused/32", || {
        for ((tape, regs), value) in tapes.iter().zip(&mut regs).zip(&mut out) {
            *value = tape.value(x, &[], &[], regs);
        }
        std::hint::black_box(&out);
    });
}
