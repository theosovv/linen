//! Hindley-Milner type inference (Algorithm W) for Linen.
//!
//! Provides let-polymorphism and unification for the type system defined in `types.rs`.

use crate::ast;
use crate::error::TypeError;
use crate::span::Span;
use crate::types::{Scheme, Type, TypeCon, TypeEnv, TypeVar};
use std::collections::HashMap;

/// A substitution mapping type variables to types.
pub type Subst = HashMap<TypeVar, Type>;

/// The type inference engine.
pub struct TypeChecker {
    var_counter: u64,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self { var_counter: 0 }
    }

    fn fresh_var(&mut self) -> TypeVar {
        let v = self.var_counter;
        self.var_counter += 1;
        v
    }

    /// Infer the type of an expression in the given environment.
    pub fn infer(&mut self, env: &TypeEnv, expr: &ast::Expr) -> Result<(Subst, Type), TypeError> {
        match expr {
            ast::Expr::Literal(lit, span) => Ok((HashMap::new(), self.lit_type(lit))),
            ast::Expr::Ident(name, span) => {
                let scheme = env
                    .lookup(name)
                    .ok_or_else(|| TypeError::UndefinedVariable {
                        name: name.clone(),
                        span: *span,
                    })?;
                Ok((HashMap::new(), self.instantiate(scheme)))
            }
            _ => {
                // TODO: implement full Algorithm W for all expression forms
                let tv = self.fresh_var();
                Ok((HashMap::new(), Type::Var(tv)))
            }
        }
    }

    /// Generalize a type into a scheme by abstracting over free variables not in the environment.
    pub fn generalize(&self, env: &TypeEnv, ty: Type) -> Scheme {
        let free_in_env: Vec<TypeVar> = env
            .bindings
            .values()
            .flat_map(|s| self.free_vars_in_scheme(s))
            .collect();
        let free_in_ty = self.free_vars(&ty);
        let vars: Vec<TypeVar> = free_in_ty
            .into_iter()
            .filter(|v| !free_in_env.contains(v))
            .collect();
        Scheme { vars, ty }
    }

    /// Instantiate a scheme with fresh type variables.
    pub fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let subst: Subst = scheme
            .vars
            .iter()
            .map(|&v| (v, Type::Var(self.fresh_var())))
            .collect();
        self.apply_subst(&subst, &scheme.ty)
    }

    /// Unify two types, producing a substitution.
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Subst, TypeError> {
        match (t1, t2) {
            (Type::Var(v), t) | (t, Type::Var(v)) => self.var_bind(*v, t),
            (Type::Con(c1, args1), Type::Con(c2, args2))
                if c1 == c2 && args1.len() == args2.len() =>
            {
                let mut subst = HashMap::new();
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    let s =
                        self.unify(&self.apply_subst(&subst, a1), &self.apply_subst(&subst, a2))?;
                    subst = self.compose_subst(&s, &subst);
                }
                Ok(subst)
            }
            (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
                let s1 = self.unify(a1, a2)?;
                let s2 = self.unify(&self.apply_subst(&s1, b1), &self.apply_subst(&s1, b2))?;
                Ok(self.compose_subst(&s2, &s1))
            }
            (Type::Tuple(ts1), Type::Tuple(ts2)) if ts1.len() == ts2.len() => {
                let mut subst = HashMap::new();
                for (t1, t2) in ts1.iter().zip(ts2.iter()) {
                    let s =
                        self.unify(&self.apply_subst(&subst, t1), &self.apply_subst(&subst, t2))?;
                    subst = self.compose_subst(&s, &subst);
                }
                Ok(subst)
            }
            _ => Err(TypeError::UnificationError(
                format!("Cannot unify {:?} with {:?}", t1, t2),
                Span::default(),
            )),
        }
    }

    fn var_bind(&self, var: TypeVar, ty: &Type) -> Result<Subst, TypeError> {
        if let Type::Var(v) = ty {
            if *v == var {
                return Ok(HashMap::new());
            }
        }
        if self.occurs_check(var, ty) {
            return Err(TypeError::InfiniteType(Span::default()));
        }
        let mut subst = HashMap::new();
        subst.insert(var, ty.clone());
        Ok(subst)
    }

    fn occurs_check(&self, var: TypeVar, ty: &Type) -> bool {
        match ty {
            Type::Var(v) => *v == var,
            Type::Con(_, args) => args.iter().any(|t| self.occurs_check(var, t)),
            Type::Arrow(a, b) => self.occurs_check(var, a) || self.occurs_check(var, b),
            Type::Tuple(ts) => ts.iter().any(|t| self.occurs_check(var, t)),
            Type::Record(fields) => fields.iter().any(|(_, t)| self.occurs_check(var, t)),
        }
    }

    fn apply_subst(&self, subst: &Subst, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => subst.get(v).cloned().unwrap_or_else(|| Type::Var(*v)),
            Type::Con(c, args) => Type::Con(
                c.clone(),
                args.iter().map(|a| self.apply_subst(subst, a)).collect(),
            ),
            Type::Arrow(a, b) => Type::Arrow(
                Box::new(self.apply_subst(subst, a)),
                Box::new(self.apply_subst(subst, b)),
            ),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.apply_subst(subst, t)).collect()),
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.apply_subst(subst, t)))
                    .collect(),
            ),
        }
    }

    fn compose_subst(&self, s1: &Subst, s2: &Subst) -> Subst {
        let mut result: Subst = s1
            .iter()
            .map(|(k, v)| (*k, self.apply_subst(s2, v)))
            .collect();
        for (k, v) in s2 {
            if !result.contains_key(k) {
                result.insert(*k, v.clone());
            }
        }
        result
    }

    fn lit_type(&self, lit: &ast::Literal) -> Type {
        match lit {
            ast::Literal::Int(_) => Type::Con(TypeCon::Int, vec![]),
            ast::Literal::Float(_) => Type::Con(TypeCon::Float, vec![]),
            ast::Literal::Bool(_) => Type::Con(TypeCon::Bool, vec![]),
            ast::Literal::String(_) => Type::Con(TypeCon::String, vec![]),
            ast::Literal::Unit => Type::Con(TypeCon::Unit, vec![]),
        }
    }

    fn free_vars(&self, ty: &Type) -> Vec<TypeVar> {
        match ty {
            Type::Var(v) => vec![*v],
            Type::Con(_, args) => args.iter().flat_map(|a| self.free_vars(a)).collect(),
            Type::Arrow(a, b) => {
                let mut vars = self.free_vars(a);
                vars.extend(self.free_vars(b));
                vars
            }
            Type::Tuple(ts) => ts.iter().flat_map(|t| self.free_vars(t)).collect(),
            Type::Record(fields) => fields.iter().flat_map(|(_, t)| self.free_vars(t)).collect(),
        }
    }

    fn free_vars_in_scheme(&self, scheme: &Scheme) -> Vec<TypeVar> {
        let mut vars = self.free_vars(&scheme.ty);
        vars.retain(|v| !scheme.vars.contains(v));
        vars
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
