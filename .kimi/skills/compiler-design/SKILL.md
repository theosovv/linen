```yaml
---
name: compiler-design
description: Compiler pipeline, AST, bytecode VM, and JIT for Linen language
---

# Compiler Design for Linen

## Pipeline Overview
Source (.ln)
↓
Tokenizer (logos) → Token stream
↓
Parser (chumsky) → Green Tree (CST) → AST
↓
Macro Expansion (compile-time)
↓
Type Checker (HM + Linear) → Typed AST
↓
IR Generation (ANF)
↓
Optimizations (constant fold, inlining)
↓
Code Generation → Bytecode (.lnc)
↓
VM / JIT Execution
plain
Copy

## AST Structure

### Core Types
```rust
pub struct Module {
    pub name: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

pub enum Declaration {
    Let(LetBinding),
    LetRec(Vec<LetBinding>),
    Type(TypeDeclaration),
    Foreign(ForeignDeclaration),
    Macro(MacroDeclaration),
}

pub enum Expr {
    Var(String),
    Literal(Literal),
    App(Box<Expr>, Vec<Expr>),
    Lambda(Vec<Parameter>, Box<Expr>),
    Let(LetBinding, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<Arm>),
    Proc(Vec<Parameter>, Vec<ProcStmt>),
    MacroInvoke { name: String, args: Vec<TokenTree> },
}
Bytecode VM
Value Representation (NaN Boxing)
rust
Copy
pub type Value = u64;

const TAG_INT: u64   = 0x7FF8_0000_0000_0000;
const TAG_BOOL: u64  = 0x7FF9_0000_0000_0000;
const TAG_OBJ: u64   = 0x7FFB_0000_0000_0000;

pub enum Object {
    String(String),
    Array(Vec<Value>),
    Closure(Closure),
    Behavior(BehaviorState),
}
Opcode Categories
Table
Range	Context	Description
0x00-0x9F	Universal	Stack, arithmetic, local vars
0xA0-0xCF	Control	FRP, closures, jumps
0xD0-0xDF	Audio	Oscillators, filters, I/O
0xE0-0xEF	Universal	SIMD operations
0xF0-0xFF	Control	Exceptions
Key Opcodes
rust
Copy
// Stack
PUSH_CONST(u32), POP, DUP, SWAP

// Arithmetic  
FADD, FSUB, FMUL, FDIV, FSIN, FCOS

// Control
JUMP(i32), JUMP_IF(i32), CALL(u8), RETURN

// FRP
BEHAVIOR_MAP, EVENT_MERGE, SF_COMPOSE

// Audio (Audio context only!)
OSC_SIN, FILTER_BIQUAD, AUDIO_OUT

// Linear types
LINEAR_MOVE, LINEAR_CLONE, LINEAR_DROP
Type Checker (HM + Linear)
Algorithm W
rust
Copy
fn infer(env: &Env, expr: &Expr) -> Result<(Subst, Type), Error> {
    match expr {
        Expr::Var(x) => {
            let ty = env.lookup(x)?;
            Ok((Subst::empty(), instantiate(ty)))
        }
        Expr::Lambda(param, body) => {
            let fresh = fresh_var();
            let env = env.extend(param, fresh.clone());
            let (s1, ty_body) = infer(&env, body)?;
            let ty = Type::Arrow(
                s1.apply(&fresh), 
                ty_body
            );
            Ok((s1, ty))
        }
        // ...
    }
}
Linear Type Checking
rust
Copy
pub struct LinearCtx {
    owned: HashSet<Value>,
    borrowed: HashMap<Value, usize>,
}

impl LinearCtx {
    fn check_move(&mut self, v: Value) -> Result<(), Error> {
        if !self.owned.remove(&v) {
            Err(Error::UseAfterMove(v))
        } else {
            Ok(())
        }
    }
    
    fn check_clone(&self, v: Value) -> Result<(), Error> {
        if self.owned.contains(&v) {
            Ok(()) // Can clone owned
        } else {
            Err(Error::CloneAfterMove(v))
        }
    }
}
JIT Compilation
Tiered Compilation
Table
Tier	Trigger	Backend	Opt Level
0	Always	Interpreter	None
1	1000+ calls	Cranelift	Basic
2	Hot loop, stable	LLVM	Aggressive
Cranelift Integration
rust
Copy
use cranelift::codegen::ir::InstBuilder;

fn compile_function(func: &Function) -> CompiledFunc {
    let mut ctx = codegen::Context::new();
    // ... build IR ...
    ctx.compile(isa).unwrap()
}
Hot Reload
State Migration
rust
Copy
pub struct StatefulNode {
    pub node_id: u64,        // Hash of AST path
    pub node_type: NodeType, // Oscillator, ADSR, etc.
    pub state_size: u32,
    pub version: u16,
}

pub trait StateMigration {
    fn can_migrate(from: u16, to: u16) -> bool;
    fn migrate(state: &[u8], from: u16, to: u16) -> Vec<u8>;
}
Testing Strategy
rust
Copy
// Unit: Parser
#[test]
fn test_parse_let() {
    let ast = parse("let x = 5 in x + 1").unwrap();
    assert_matches!(ast, Expr::Let(...));
}

// Unit: Type checker
#[test]
fn test_linear_violation() {
    let result = check("let s = sinOsc 440 in (s, s)");
    assert_matches!(result, Err(LinearError::DoubleMove));
}

// Integration: VM execution
#[test]
fn test_vm_sine() {
    let bytecode = compile("sinOsc 440.0");
    let mut vm = VM::new();
    let result = vm.run(&bytecode);
    assert!(result.is_behavior());
}
Resources
"Types and Programming Languages" by Pierce
"Crafting Interpreters" by Nystrom
Cranelift docs: https://docs.rs/cranelift/
LALRPOP book: https://lalrpop.github.io/