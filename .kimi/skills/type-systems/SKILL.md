```yaml
---
name: type-systems
description: Hindley-Milner type inference, linear types, and FRP type system for Linen
---

# Type Systems for Linen

## Hindley-Milner

### Basic Types
```linen
Int         -- 64-bit integer
Float       -- 64-bit float
Bool        -- true | false
String      -- UTF-8 text
()          -- Unit

a -> b      -- Function
(a, b)      -- Tuple
[a]         -- List
Polymorphism
linen
Copy
-- Implicit forall: identity works for any type
identity : a -> a
identity x = x

-- Explicit forall (internal representation)
identity : forall a. a -> a

-- Multiple type variables
map : (a -> b) -> [a] -> [b]
map f [] = []
map f (x:xs) = f x : map f xs
Type Constraints
linen
Copy
-- Num: types that support arithmetic
sum : Num a => [a] -> a
sum [] = 0
sum (x:xs) = x + sum xs

-- Eq: types that support equality
member : Eq a => a -> [a] -> Bool
member x [] = False
member x (y:ys) = x == y || member x ys

-- Multiple constraints
func : (Num a, Eq a) => a -> a -> Bool
FRP Types
Behavior (Continuous)
linen
Copy
-- Behavior is a functor, applicative, monad
Behavior Float        -- Time-varying float
Behavior (Float, Float)  -- Stereo signal

-- Lifting
pure 5 : Behavior Int  -- Constant
liftA2 (+) :: Behavior Float -> Behavior Float -> Behavior Float
Event (Discrete)
linen
Copy
Event ()              -- Trigger with no value
Event Float           -- Event carrying float
Event (Note, Velocity) -- MIDI note on

-- Event is a functor, but not monad (no join)
mapE : (a -> b) -> Event a -> Event b
Signal Function
linen
Copy
SF a b  -- Stateful transformation from a to b

-- Category instance
identity : SF a a
(>>>) : SF a b -> SF b c -> SF a c

-- Arrow instance
arr : (a -> b) -> SF a b
first : SF a b -> SF (a, c) (b, c)
loop : SF (a, c) (b, c) -> SF a b
Linear Types
Motivation
Resources that must be used exactly once:
Audio buffers (free after use)
File handles
Network connections
Syntax
linen
Copy
-- Linear annotation (prefix 'lin')
allocBuffer : Int -> lin AudioBuffer
freeBuffer : lin AudioBuffer -> ()

-- Type error: use after move
bad = 
    let buf = allocBuffer 1024 in
    let x = buf in      -- move buf to x
    freeBuffer buf      -- ERROR: buf already moved

-- Correct: use exactly once
good = 
    let buf = allocBuffer 1024 in
    process buf |> freeBuffer
Operations
linen
Copy
-- Explicit clone (requires Clone trait)
clone : lin a -> a  -- a must implement Clone

-- Explicit drop (usually automatic)
drop : lin a -> ()

-- Borrow (temporary, non-owning reference)
borrow : lin a -> &a
Runtime Checking (Debug)
rust
Copy
// In VM (debug builds only)
struct LinearChecker {
    owned: HashSet<Value>,
}

impl LinearChecker {
    fn verify_move(&mut self, v: Value) {
        assert!(self.owned.remove(&v), "Use after move");
    }
    
    fn verify_drop(&self, v: Value) {
        assert!(self.owned.contains(&v), "Drop of non-owned");
    }
}
Type Inference Algorithm
Algorithm W
rust
Copy
fn infer(env: &Env, expr: &Expr) -> Result<(Subst, Type), Error> {
    match expr {
        Expr::Var(x) => {
            let scheme = env.lookup(x)?;
            Ok((Subst::empty(), instantiate(scheme)))
        }
        
        Expr::App(f, arg) => {
            let (s1, ty_f) = infer(env, f)?;
            let (s2, ty_arg) = infer(&env.apply(&s1), arg)?;
            let ty_result = fresh_var();
            let s3 = unify(&ty_f, &Type::Arrow(ty_arg, ty_result.clone()))?;
            Ok((s3.compose(s2).compose(s1), s3.apply(&ty_result)))
        }
        
        Expr::Lambda(x, body) => {
            let ty_x = fresh_var();
            let env = env.extend(x, Scheme::mono(ty_x.clone()));
            let (s, ty_body) = infer(&env, body)?;
            Ok((s.clone(), Type::Arrow(s.apply(&ty_x), ty_body)))
        }
        
        Expr::Let(x, val, body) => {
            let (s1, ty_val) = infer(env, val)?;
            let scheme = generalize(&env.apply(&s1), ty_val);
            let env = env.extend(x, scheme);
            let (s2, ty_body) = infer(&env, body)?;
            Ok((s2.compose(s1), ty_body))
        }
    }
}
Unification
rust
Copy
fn unify(t1: &Type, t2: &Type) -> Result<Subst, Error> {
    match (t1, t2) {
        (Type::Var(a), t) | (t, Type::Var(a)) => 
            if occurs_check(a, t) {
                Err(Error::InfiniteType)
            } else {
                Ok(Subst::singleton(a.clone(), t.clone()))
            }
        
        (Type::Con(c1, args1), Type::Con(c2, args2)) if c1 == c2 => {
            unify_many(args1, args2)
        }
        
        (Type::Arrow(a1, r1), Type::Arrow(a2, r2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(r1), &s1.apply(r2))?;
            Ok(s2.compose(s1))
        }
        
        _ => Err(Error::UnificationFailed(t1.clone(), t2.clone()))
    }
}
Kind System (Advanced)
linen
Copy
-- Behaviors are indexed by time representation
Behavior @Audio Float     -- Audio-rate (48kHz)
Behavior @Control Float   -- Control-rate (1kHz)

-- Type-level naturals for buffer sizes
Vec @1024 Float  -- Vector of exactly 1024 floats
Resources
"Types and Programming Languages" by Benjamin Pierce
"Linear Logic" by Jean-Yves Girard
"Rust Ownership and Borrowing" (for linear types practice)
"Typing Haskell in Haskell" by Simon Peyton Jones