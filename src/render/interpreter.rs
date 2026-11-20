use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode, SourceSpan,
    dsp::Sine,
    sema::{BindingId, Type, TypedExpr, TypedProgram},
    syntax::BinaryOp,
};

pub struct NaiveInterpreter<'program> {
    program: &'program TypedProgram,
    bindings: Vec<EvalBinding>,
    oscillators: Vec<Sine>,
    values: Vec<Option<EvalValue>>,
}

struct EvalBinding {
    id: BindingId,
    value: EvalExpr,
}

struct EvalExpr {
    kind: EvalExprKind,
    ty: Type,
    span: SourceSpan,
}

enum EvalExprKind {
    Number(f64),
    Binding(BindingId),
    Sine {
        frequency: Box<EvalExpr>,
        state: usize,
    },
    Unary {
        operand: Box<EvalExpr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<EvalExpr>,
        right: Box<EvalExpr>,
    },
    Group(Box<EvalExpr>),
    Lift(Box<EvalExpr>),
}

#[derive(Clone, Copy)]

enum EvalValue {
    Float(f64),
    Sig(f32),
}

fn lower_expression(expression: &TypedExpr, oscillators: &mut Vec<Sine>) -> EvalExpr {
    let kind = match &expression.kind {
        crate::sema::TypedExprKind::Number(value) => EvalExprKind::Number(*value),
        crate::sema::TypedExprKind::Identifier { id, identifier: _ } => EvalExprKind::Binding(*id),
        crate::sema::TypedExprKind::Call {
            builtin: _,
            callee: _,
            arguments,
        } => {
            let frequency = lower_expression(&arguments[0], oscillators);
            let state = oscillators.len();

            oscillators.push(Sine::new());

            EvalExprKind::Sine {
                frequency: Box::new(frequency),
                state,
            }
        }
        crate::sema::TypedExprKind::Unary {
            operator: _,
            operand,
        } => {
            let operand = lower_expression(operand, oscillators);

            EvalExprKind::Unary {
                operand: Box::new(operand),
            }
        }
        crate::sema::TypedExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = lower_expression(left, oscillators);
            let right = lower_expression(right, oscillators);

            EvalExprKind::Binary {
                operator: *operator,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        crate::sema::TypedExprKind::Group(typed_expr) => {
            let inner = lower_expression(typed_expr, oscillators);

            EvalExprKind::Group(Box::new(inner))
        }
        crate::sema::TypedExprKind::LiftFloatToSig(typed_expr) => {
            let x = lower_expression(typed_expr, oscillators);

            EvalExprKind::Lift(Box::new(x))
        }
    };

    EvalExpr {
        kind,
        ty: expression.ty,
        span: expression.span,
    }
}

fn invalid_value_type(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::TypeMismatch,
        "runtime value does not match its typed expression",
        DiagnosticLabel::new(span, "unexpected runtime value type"),
    )
}

fn float_value(value: EvalValue, span: SourceSpan) -> Result<f64, Diagnostic> {
    match value {
        EvalValue::Float(value) => Ok(value),
        EvalValue::Sig(_) => Err(invalid_value_type(span)),
    }
}

fn signal_value(value: EvalValue, span: SourceSpan) -> Result<f32, Diagnostic> {
    match value {
        EvalValue::Sig(value) => Ok(value),
        EvalValue::Float(_) => Err(invalid_value_type(span)),
    }
}

fn eval(
    expression: &EvalExpr,
    values: &[Option<EvalValue>],
    oscillators: &mut [Sine],
    sample_index: u64,
    sample_rate: u32,
) -> Result<EvalValue, Diagnostic> {
    match &expression.kind {
        EvalExprKind::Number(value) => Ok(EvalValue::Float(*value)),
        EvalExprKind::Binding(id) => values
            .get(id.index())
            .copied()
            .flatten()
            .ok_or_else(|| invalid_value_type(expression.span)),
        EvalExprKind::Sine { frequency, state } => {
            let frequency_value = eval(frequency, values, oscillators, sample_index, sample_rate)?;
            let frequency_value = signal_value(frequency_value, frequency.span)?;

            let oscillator = oscillators
                .get_mut(*state)
                .ok_or_else(|| invalid_value_type(expression.span))?;

            oscillator
                .next_sample(f64::from(frequency_value), sample_rate)
                .map(EvalValue::Sig)
                .map_err(|_| {
                    Diagnostic::error(
                        ErrorCode::NonFiniteOutput,
                        "sine frequency contains NaN or infinity",
                        DiagnosticLabel::new(
                            frequency.span,
                            "this argument produces the invalid frequency",
                        ),
                    )
                    .with_sample_index(sample_index)
                })
        }
        EvalExprKind::Unary { operand } => {
            let operand = eval(operand, values, oscillators, sample_index, sample_rate)?;

            match expression.ty {
                Type::Float => {
                    let value = float_value(operand, expression.span)?;

                    Ok(EvalValue::Float(-value))
                }
                Type::Sig => {
                    let value = signal_value(operand, expression.span)?;

                    Ok(EvalValue::Sig(-value))
                }
            }
        }
        EvalExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = eval(left, values, oscillators, sample_index, sample_rate)?;
            let right = eval(right, values, oscillators, sample_index, sample_rate)?;

            match expression.ty {
                Type::Float => {
                    let left = float_value(left, expression.span)?;
                    let right = float_value(right, expression.span)?;
                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };

                    Ok(EvalValue::Float(value))
                }
                Type::Sig => {
                    let left = signal_value(left, expression.span)?;
                    let right = signal_value(right, expression.span)?;

                    let value = match operator {
                        BinaryOp::Add => left + right,
                        BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right,
                        BinaryOp::Divide => left / right,
                    };

                    Ok(EvalValue::Sig(value))
                }
            }
        }
        EvalExprKind::Group(inner) => eval(inner, values, oscillators, sample_index, sample_rate),
        EvalExprKind::Lift(inner) => {
            let value = eval(inner, values, oscillators, sample_index, sample_rate)?;
            let value = float_value(value, expression.span)?;

            Ok(EvalValue::Sig(value as f32))
        }
    }
}

impl<'program> NaiveInterpreter<'program> {
    pub fn new(program: &'program TypedProgram) -> Self {
        let mut oscillators = Vec::new();
        let mut bindings = Vec::new();

        for binding in &program.bindings {
            bindings.push(EvalBinding {
                id: binding.id,
                value: lower_expression(&binding.value, &mut oscillators),
            });
        }

        let value_count = bindings
            .iter()
            .map(|binding| binding.id.index() + 1)
            .max()
            .unwrap_or(0);

        let values = vec![None; value_count];

        Self {
            program,
            bindings,
            oscillators,
            values,
        }
    }

    pub fn reset(&mut self) {
        for oscillator in &mut self.oscillators {
            oscillator.reset();
        }
    }

    pub fn next_sample(&mut self, sample_index: u64, sample_rate: u32) -> Result<f32, Diagnostic> {
        for binding in &self.bindings {
            let value = eval(
                &binding.value,
                &self.values,
                &mut self.oscillators,
                sample_index,
                sample_rate,
            )?;

            self.values[binding.id.index()] = Some(value);
        }

        match self
            .values
            .get(self.program.output.index())
            .copied()
            .flatten()
        {
            Some(EvalValue::Sig(sample)) => Ok(sample),
            Some(EvalValue::Float(_)) | None => Err(invalid_value_type(self.program.span)),
        }
    }
}
