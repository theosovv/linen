//! Tree-walking interpreter for Linen Core AST.
//!
//! Evaluates Core expressions to Values. Used by both REPL and `linc run`.

use crate::core_lang::{Atom, CoreExpr, CoreModule, CorePattern};
use crate::value::Value;
use std::collections::HashMap;

/// Evaluation environment.
pub type Env = HashMap<String, Value>;

/// Evaluate a Core module, returning the top-level environment.
pub fn eval_module(module: &CoreModule) -> Env {
    let mut env = Env::new();
    for (name, expr) in &module.bindings {
        let value = eval_expr(expr, &env);
        env.insert(name.clone(), value);
    }
    env
}

/// Evaluate a Core expression in the given environment.
pub fn eval_expr(expr: &CoreExpr, env: &Env) -> Value {
    match expr {
        CoreExpr::Atom(atom, _) => eval_atom(atom, env),
        CoreExpr::Let(name, value, body, _) => {
            let val = eval_expr(value, env);
            let mut new_env = env.clone();
            new_env.insert(name.clone(), val);
            eval_expr(body, &new_env)
        }
        CoreExpr::Apply(func, args, _) => {
            let func_val = eval_atom(func, env);
            let arg_vals: Vec<Value> = args.iter().map(|a| eval_atom(a, env)).collect();
            apply_value(func_val, &arg_vals)
        }
        CoreExpr::Lambda(params, body, _) => Value::Closure {
            params: params.clone(),
            body: body.as_ref().clone(),
            env: env.clone(),
        },
        CoreExpr::If(cond, then_branch, else_branch, _) => match eval_atom(cond, env) {
            Value::Literal(crate::ast::Literal::Bool(true)) => eval_expr(then_branch, env),
            Value::Literal(crate::ast::Literal::Bool(false)) => eval_expr(else_branch, env),
            other => panic!("Expected boolean in if condition, got {:?}", other),
        },
        CoreExpr::Match(scrut, arms, _) => {
            let scrut_val = eval_atom(scrut, env);
            for (pat, branch) in arms {
                if let Some(bindings) = match_pattern(pat, &scrut_val) {
                    let mut new_env = env.clone();
                    new_env.extend(bindings);
                    return eval_expr(branch, &new_env);
                }
            }
            panic!("Non-exhaustive pattern match for value {:?}", scrut_val)
        }
    }
}

fn eval_atom(atom: &Atom, env: &Env) -> Value {
    match atom {
        Atom::Var(name) => env
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Undefined variable: {}", name)),
        Atom::Literal(lit) => Value::from_literal(lit.clone()),
    }
}

fn apply_value(func: Value, args: &[Value]) -> Value {
    match func {
        Value::Closure { params, body, env } => {
            assert_eq!(
                params.len(),
                args.len(),
                "Arity mismatch: expected {}, got {}",
                params.len(),
                args.len()
            );
            let mut new_env = env;
            for (p, a) in params.iter().zip(args.iter()) {
                new_env.insert(p.clone(), a.clone());
            }
            eval_expr(&body, &new_env)
        }
        Value::NativeFn(f) => f(args),
        other => panic!("Expected function, got {:?}", other),
    }
}

fn match_pattern(pat: &CorePattern, value: &Value) -> Option<Env> {
    match (pat, value) {
        (CorePattern::Wildcard, _) => Some(Env::new()),
        (CorePattern::Var(name), val) => {
            let mut env = Env::new();
            env.insert(name.clone(), val.clone());
            Some(env)
        }
        (CorePattern::Literal(l1), Value::Literal(l2)) => {
            if l1 == l2 {
                Some(Env::new())
            } else {
                None
            }
        }
        (CorePattern::Tuple(pats), Value::Tuple(vals)) if pats.len() == vals.len() => {
            let mut env = Env::new();
            for (p, v) in pats.iter().zip(vals.iter()) {
                let bindings = match_pattern(p, v)?;
                env.extend(bindings);
            }
            Some(env)
        }
        (CorePattern::Variant(name1, pats), Value::Variant(name2, vals))
            if name1 == name2 && pats.len() == vals.len() =>
        {
            let mut env = Env::new();
            for (p, v) in pats.iter().zip(vals.iter()) {
                let bindings = match_pattern(p, v)?;
                env.extend(bindings);
            }
            Some(env)
        }
        (CorePattern::Record(pats), Value::Record(fields)) => {
            let mut env = Env::new();
            for (name, pat) in pats {
                let val = fields.get(name)?;
                let bindings = match_pattern(pat, val)?;
                env.extend(bindings);
            }
            Some(env)
        }
        _ => None,
    }
}
