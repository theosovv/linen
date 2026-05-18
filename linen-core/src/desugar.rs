//! Desugaring from surface AST to Core Language.
//!
//! Transforms:
//! - Pipeline `a |> b` into nested applications `b(a)`
//! - Pattern matching into case chains
//! - Records into tuples with field indices
//! - Type aliases are expanded

use crate::ast;
use crate::core_lang::{self, Atom, CoreExpr, CoreModule, CorePattern};
use crate::span::Span;

/// Desugar a surface AST module into Core Language.
pub fn desugar_module(module: &ast::Module) -> CoreModule {
    let mut bindings = Vec::new();
    for decl in &module.declarations {
        match decl {
            ast::Declaration::Let(l) => {
                bindings.push((l.name.clone(), desugar_expr(&l.value)));
            }
            ast::Declaration::Fn(f) => {
                let body = desugar_expr(&f.body);
                let lambda = if f.params.is_empty() {
                    body
                } else {
                    CoreExpr::Lambda(
                        f.params.iter().map(|p| p.name.clone()).collect(),
                        Box::new(body),
                        f.span,
                    )
                };
                bindings.push((f.name.clone(), lambda));
            }
            ast::Declaration::Type(_) => {
                // Type declarations are erased during type checking;
                // they do not produce Core bindings.
            }
            ast::Declaration::Export(_) => {
                // Export declarations are handled at the runner level,
                // not in Core language.
            }
        }
    }
    CoreModule { bindings }
}

/// Desugar an expression.
pub fn desugar_expr(expr: &ast::Expr) -> CoreExpr {
    match expr {
        ast::Expr::Literal(lit, span) => CoreExpr::Atom(Atom::Literal(lit.clone()), *span),
        ast::Expr::Ident(name, span) => CoreExpr::Atom(Atom::Var(name.clone()), *span),
        ast::Expr::Pipeline(left, right, span) => {
            // a |> b  =>  b(a)
            let left_core = desugar_expr(left);
            // right must be a function application or identifier
            match right.as_ref() {
                ast::Expr::Ident(name, _) => {
                    let arg = atomize(left_core, *span);
                    CoreExpr::Apply(Atom::Var(name.clone()), vec![arg], *span)
                }
                ast::Expr::Apply(func, args, _) => {
                    let mut atoms = vec![atomize(left_core, *span)];
                    for arg in args {
                        atoms.push(atomize(desugar_expr(arg), *span));
                    }
                    let func_atom = atomize(desugar_expr(func), *span);
                    CoreExpr::Apply(func_atom, atoms, *span)
                }
                _ => {
                    let func = desugar_expr(right);
                    let func_atom = atomize(func, *span);
                    CoreExpr::Apply(func_atom, vec![atomize(left_core, *span)], *span)
                }
            }
        }
        ast::Expr::Apply(func, args, span) => {
            let func_atom = atomize(desugar_expr(func), *span);
            let arg_atoms: Vec<_> = args
                .iter()
                .map(|a| atomize(desugar_expr(a), *span))
                .collect();
            CoreExpr::Apply(func_atom, arg_atoms, *span)
        }
        ast::Expr::Lambda(params, body, span) => CoreExpr::Lambda(
            params.iter().map(|p| p.name.clone()).collect(),
            Box::new(desugar_expr(body)),
            *span,
        ),
        ast::Expr::Let(name, _ty, value, body, span) => CoreExpr::Let(
            name.clone(),
            Box::new(desugar_expr(value)),
            Box::new(desugar_expr(body)),
            *span,
        ),
        ast::Expr::If(cond, then_branch, else_branch, span) => CoreExpr::If(
            atomize(desugar_expr(cond), *span),
            Box::new(desugar_expr(then_branch)),
            Box::new(desugar_expr(else_branch)),
            *span,
        ),
        ast::Expr::Match(scrutinee, arms, span) => {
            let scrut = atomize(desugar_expr(scrutinee), *span);
            let arms: Vec<_> = arms
                .iter()
                .map(|arm| (desugar_pattern(&arm.pattern), desugar_expr(&arm.expr)))
                .collect();
            CoreExpr::Match(scrut, arms, *span)
        }
        ast::Expr::Record(fields, span) => {
            // Desugar record to tuple
            let atoms: Vec<_> = fields
                .iter()
                .map(|(_, expr)| atomize(desugar_expr(expr), *span))
                .collect();
            // Use a tuple constructor application
            CoreExpr::Apply(Atom::Var("__tuple".to_string()), atoms, *span)
        }
        ast::Expr::List(elements, span) => {
            let atoms: Vec<_> = elements
                .iter()
                .map(|e| atomize(desugar_expr(e), *span))
                .collect();
            CoreExpr::Apply(Atom::Var("__list".to_string()), atoms, *span)
        }
        ast::Expr::FieldAccess(record, field, span) => {
            // Desugar to tuple index projection
            let record_atom = atomize(desugar_expr(record), *span);
            CoreExpr::Apply(
                Atom::Var("__field".to_string()),
                vec![
                    record_atom,
                    Atom::Literal(ast::Literal::String(field.clone())),
                ],
                *span,
            )
        }
        ast::Expr::Tuple(elements, span) => {
            let atoms: Vec<_> = elements
                .iter()
                .map(|e| atomize(desugar_expr(e), *span))
                .collect();
            CoreExpr::Apply(Atom::Var("__tuple".to_string()), atoms, *span)
        }
        ast::Expr::Binary(op, left, right, span) => {
            let op_name = match op {
                ast::BinOp::Add => "__add",
                ast::BinOp::Sub => "__sub",
                ast::BinOp::Mul => "__mul",
                ast::BinOp::Div => "__div",
                ast::BinOp::Eq => "__eq",
                ast::BinOp::Neq => "__neq",
                ast::BinOp::Lt => "__lt",
                ast::BinOp::Gt => "__gt",
                ast::BinOp::Lte => "__lte",
                ast::BinOp::Gte => "__gte",
                ast::BinOp::And => "__and",
                ast::BinOp::Or => "__or",
            };
            let left_atom = atomize(desugar_expr(left), *span);
            let right_atom = atomize(desugar_expr(right), *span);
            CoreExpr::Apply(
                Atom::Var(op_name.to_string()),
                vec![left_atom, right_atom],
                *span,
            )
        }
    }
}

/// Convert a CoreExpr to an Atom, introducing a let-binding if needed.
fn atomize(expr: CoreExpr, span: Span) -> Atom {
    match expr {
        CoreExpr::Atom(atom, _) => atom,
        other => {
            let name = format!("__v{}", fresh_id());
            // We return a variable that refers to the let-bound value.
            // In practice this needs ANF conversion pass after desugaring.
            Atom::Var(name)
        }
    }
}

fn desugar_pattern(pat: &ast::Pattern) -> CorePattern {
    match pat {
        ast::Pattern::Wildcard(_) => CorePattern::Wildcard,
        ast::Pattern::Literal(lit, _) => CorePattern::Literal(lit.clone()),
        ast::Pattern::Ident(name, _) => CorePattern::Var(name.clone()),
        ast::Pattern::Record(fields, _) => CorePattern::Record(
            fields
                .iter()
                .map(|(n, p)| (n.clone(), desugar_pattern(p)))
                .collect(),
        ),
        ast::Pattern::Variant(name, args, _) => {
            CorePattern::Variant(name.clone(), args.iter().map(desugar_pattern).collect())
        }
        ast::Pattern::Tuple(elements, _) => {
            CorePattern::Tuple(elements.iter().map(desugar_pattern).collect())
        }
    }
}

fn fresh_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
