use linen::sema::{
    BindingId, BuiltinId, Type, TypedBinding, TypedExpr, TypedExprKind, TypedProgram,
};
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
    slot_count: usize,
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
    Block {
        bindings: Vec<ReferenceBinding>,
        result: Box<ReferenceExpr>,
    },
    Lift(Box<ReferenceExpr>),
}

#[derive(Clone, Copy)]
enum ReferenceValue {
    Float(f64),
    Signal(f32),
}

struct ReferenceBuilder {
    phases: Vec<f64>,
    slot_count: usize,
}

impl ReferenceBuilder {
    fn register(&mut self, id: BindingId) {
        self.slot_count = self.slot_count.max(id.index() + 1);
    }
}

impl ReferenceInterpreter {
    pub fn new(program: &TypedProgram) -> Self {
        let mut builder = ReferenceBuilder {
            phases: Vec::new(),
            slot_count: 0,
        };
        let bindings = program
            .bindings()
            .into_iter()
            .map(|binding| {
                builder.register(binding.id);
                ReferenceBinding {
                    id: binding.id,
                    value: lower_expr(&binding.value, program, &mut builder),
                }
            })
            .collect();
        builder.register(program.output);
        Self {
            bindings,
            output: program.output,
            phases: builder.phases,
            slot_count: builder.slot_count,
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
        let mut values = vec![None; self.slot_count];

        for index in 0..self.bindings.len() {
            let id = self.bindings[index].id;
            let value = eval(
                &self.bindings[index].value,
                &mut values,
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

fn lower_bindings(
    bindings: &[TypedBinding],
    program: &TypedProgram,
    builder: &mut ReferenceBuilder,
) -> Vec<ReferenceBinding> {
    bindings
        .iter()
        .map(|binding| {
            builder.register(binding.id);
            ReferenceBinding {
                id: binding.id,
                value: lower_expr(&binding.value, program, builder),
            }
        })
        .collect()
}

fn lower_expr(
    expr: &TypedExpr,
    program: &TypedProgram,
    builder: &mut ReferenceBuilder,
) -> ReferenceExpr {
    let kind = match &expr.kind {
        TypedExprKind::Number(value) => ReferenceExprKind::Number(*value),
        TypedExprKind::Identifier { id, .. } => {
            builder.register(*id);
            ReferenceExprKind::Identifier(*id)
        }
        TypedExprKind::BuiltinCall {
            builtin, arguments, ..
        } => {
            assert_eq!(*builtin, BuiltinId::SINE, "unsupported reference builtin");
            assert_eq!(arguments.len(), 1, "sine must have one checked argument");
            let frequency = Box::new(lower_expr(&arguments[0], program, builder));
            let state = builder.phases.len();
            builder.phases.push(0.0);
            ReferenceExprKind::Sine { frequency, state }
        }
        TypedExprKind::FunctionCall {
            function,
            arguments,
            ..
        } => {
            let definition = program
                .functions()
                .find(|candidate| candidate.id == *function)
                .expect("a resolved function must have a typed definition");
            assert_eq!(
                arguments.len(),
                definition.parameters.len(),
                "function arity must be checked before reference construction"
            );
            let mut bindings = Vec::with_capacity(arguments.len());
            for (argument, parameter) in arguments.iter().zip(&definition.parameters) {
                builder.register(parameter.id);
                let value = lower_expr(argument, program, builder);
                bindings.push(ReferenceBinding {
                    id: parameter.id,
                    value,
                });
            }
            let result = Box::new(lower_expr(&definition.body, program, builder));
            ReferenceExprKind::Block { bindings, result }
        }
        TypedExprKind::Unary { operator, operand } => ReferenceExprKind::Unary {
            operator: *operator,
            operand: Box::new(lower_expr(operand, program, builder)),
        },
        TypedExprKind::Binary {
            operator,
            left,
            right,
        } => ReferenceExprKind::Binary {
            operator: *operator,
            left: Box::new(lower_expr(left, program, builder)),
            right: Box::new(lower_expr(right, program, builder)),
        },
        TypedExprKind::Group(inner) => {
            ReferenceExprKind::Group(Box::new(lower_expr(inner, program, builder)))
        }
        TypedExprKind::Block { bindings, result } => ReferenceExprKind::Block {
            bindings: lower_bindings(bindings, program, builder),
            result: Box::new(lower_expr(result, program, builder)),
        },
        TypedExprKind::LiftFloatToSig(inner) => {
            ReferenceExprKind::Lift(Box::new(lower_expr(inner, program, builder)))
        }
    };
    ReferenceExpr { kind, ty: expr.ty }
}

fn eval(
    expr: &ReferenceExpr,
    values: &mut [Option<ReferenceValue>],
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
        ReferenceExprKind::Block { bindings, result } => {
            let mut saved = Vec::with_capacity(bindings.len());
            for binding in bindings {
                let value = eval(&binding.value, values, phases, sample_index, sample_rate)?;
                saved.push((binding.id, values[binding.id.index()]));
                values[binding.id.index()] = Some(value);
            }
            let result = eval(result, values, phases, sample_index, sample_rate);
            for (id, value) in saved.into_iter().rev() {
                values[id.index()] = value;
            }
            result
        }
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
