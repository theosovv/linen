use linen::sema::{BindingId, BuiltinId, Type, TypedExpr, TypedExprKind, TypedProgram};
use linen::syntax::{BinaryOp, UnaryOp};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceErrorKind {
    NonFiniteFrequency,
    NonFiniteOutput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceError {
    pub kind: ReferenceErrorKind,
    pub sample_index: u64,
}

pub struct ReferenceInterpreter {
    bindings: Vec<ReferenceBinding>,
    output: BindingId,
    phases: Vec<f64>,
}

struct ReferenceBinding {
    id: BindingId,
    value: ReferenceExpr,
}

struct ReferenceExpr {
    kind: ReferenceExprKind,
    ty: Type,
}

enum ReferenceExprKind {
    Number(f64),
    Identifier(BindingId),
    Sine {
        frequency: Box<ReferenceExpr>,
        state: usize,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<ReferenceExpr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<ReferenceExpr>,
        right: Box<ReferenceExpr>,
    },
    Group(Box<ReferenceExpr>),
    Lift(Box<ReferenceExpr>),
}

#[derive(Clone, Copy)]
enum ReferenceValue {
    Float(f64),
    Signal(f32),
}

impl ReferenceInterpreter {
    pub fn new(program: &TypedProgram) -> Self {
        let mut phases = Vec::new();
        let bindings = program
            .bindings
            .iter()
            .map(|binding| ReferenceBinding {
                id: binding.id,
                value: lower_expr(&binding.value, &mut phases),
            })
            .collect();
        Self {
            bindings,
            output: program.output,
            phases,
        }
    }

    pub fn reset(&mut self) {
        self.phases.fill(0.0);
    }

    pub fn next_sample(
        &mut self,
        sample_index: u64,
        sample_rate: u32,
    ) -> Result<f32, ReferenceError> {
        let slot_count = self
            .bindings
            .iter()
            .map(|binding| binding.id.index() + 1)
            .max()
            .unwrap_or(0);
        let mut values = vec![None; slot_count];

        for index in 0..self.bindings.len() {
            let id = self.bindings[index].id;
            let value = eval(
                &self.bindings[index].value,
                &values,
                &mut self.phases,
                sample_index,
                sample_rate,
            )?;
            values[id.index()] = Some(value);
        }

        let output = values[self.output.index()]
            .expect("the output binding must be evaluated before output");
        let ReferenceValue::Signal(sample) = output else {
            panic!("typed output must be a signal after its explicit lift");
        };
        if !sample.is_finite() {
            return Err(ReferenceError {
                kind: ReferenceErrorKind::NonFiniteOutput,
                sample_index,
            });
        }
        Ok(sample)
    }
}

fn lower_expr(expr: &TypedExpr, phases: &mut Vec<f64>) -> ReferenceExpr {
    let kind = match &expr.kind {
        TypedExprKind::Number(value) => ReferenceExprKind::Number(*value),
        TypedExprKind::Identifier { id, .. } => ReferenceExprKind::Identifier(*id),
        TypedExprKind::Call {
            builtin, arguments, ..
        } => {
            assert_eq!(*builtin, BuiltinId::SINE, "unsupported reference builtin");
            assert_eq!(arguments.len(), 1, "sine must have one checked argument");
            let state = phases.len();
            phases.push(0.0);
            ReferenceExprKind::Sine {
                frequency: Box::new(lower_expr(&arguments[0], phases)),
                state,
            }
        }
        TypedExprKind::Unary { operator, operand } => ReferenceExprKind::Unary {
            operator: *operator,
            operand: Box::new(lower_expr(operand, phases)),
        },
        TypedExprKind::Binary {
            operator,
            left,
            right,
        } => ReferenceExprKind::Binary {
            operator: *operator,
            left: Box::new(lower_expr(left, phases)),
            right: Box::new(lower_expr(right, phases)),
        },
        TypedExprKind::Group(inner) => {
            ReferenceExprKind::Group(Box::new(lower_expr(inner, phases)))
        }
        TypedExprKind::LiftFloatToSig(inner) => {
            ReferenceExprKind::Lift(Box::new(lower_expr(inner, phases)))
        }
    };
    ReferenceExpr { kind, ty: expr.ty }
}

fn eval(
    expr: &ReferenceExpr,
    values: &[Option<ReferenceValue>],
    phases: &mut [f64],
    sample_index: u64,
    sample_rate: u32,
) -> Result<ReferenceValue, ReferenceError> {
    match &expr.kind {
        ReferenceExprKind::Number(value) => Ok(ReferenceValue::Float(*value)),
        ReferenceExprKind::Identifier(id) => {
            Ok(values[id.index()].expect("resolved binding must be evaluated before use"))
        }
        ReferenceExprKind::Sine { frequency, state } => {
            let frequency = match eval(frequency, values, phases, sample_index, sample_rate)? {
                ReferenceValue::Signal(value) => f64::from(value),
                ReferenceValue::Float(_) => panic!("sine argument must be lifted to Sig"),
            };
            if !frequency.is_finite() {
                return Err(ReferenceError {
                    kind: ReferenceErrorKind::NonFiniteFrequency,
                    sample_index,
                });
            }
            let phase = phases[*state];
            let sample = (std::f64::consts::TAU * phase).sin() as f32;
            phases[*state] = (phase + frequency / f64::from(sample_rate)).rem_euclid(1.0);
            Ok(ReferenceValue::Signal(sample))
        }
        ReferenceExprKind::Unary { operator, operand } => {
            let operand = eval(operand, values, phases, sample_index, sample_rate)?;
            match (operator, operand) {
                (UnaryOp::Negate, ReferenceValue::Float(value)) => {
                    Ok(ReferenceValue::Float(-value))
                }
                (UnaryOp::Negate, ReferenceValue::Signal(value)) => {
                    Ok(ReferenceValue::Signal(-value))
                }
            }
        }
        ReferenceExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = eval(left, values, phases, sample_index, sample_rate)?;
            let right = eval(right, values, phases, sample_index, sample_rate)?;
            match expr.ty {
                Type::Float => {
                    let left = float_value(left);
                    let right = float_value(right);
                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };
                    Ok(ReferenceValue::Float(value))
                }
                Type::Sig => {
                    let left = signal_value(left);
                    let right = signal_value(right);
                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };
                    Ok(ReferenceValue::Signal(value))
                }
            }
        }
        ReferenceExprKind::Group(inner) => eval(inner, values, phases, sample_index, sample_rate),
        ReferenceExprKind::Lift(inner) => {
            let value = float_value(eval(inner, values, phases, sample_index, sample_rate)?);
            Ok(ReferenceValue::Signal(value as f32))
        }
    }
}

fn float_value(value: ReferenceValue) -> f64 {
    match value {
        ReferenceValue::Float(value) => value,
        ReferenceValue::Signal(_) => panic!("a Float expression cannot contain a runtime signal"),
    }
}

fn signal_value(value: ReferenceValue) -> f32 {
    match value {
        ReferenceValue::Signal(value) => value,
        ReferenceValue::Float(_) => panic!("mixed signals must use an explicit lift"),
    }
}
