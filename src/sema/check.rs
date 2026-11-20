use std::collections::{BTreeMap, HashMap};

use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode,
    source::SourceSpan,
    syntax::{BinaryOp, Expr, ExprKind, LetDecl, Program, RenderDecl, TypeName},
};

use super::{
    BindingId, SymbolStorage, Type, TypedBinding, TypedExpr, TypedExprKind, TypedProgram,
    TypedRender, builtin_by_name,
};

pub fn check_program(
    program: &Program,
    symbols: &mut SymbolStorage,
    sample_rate: u32,
) -> Result<TypedProgram, Diagnostic> {
    symbols.clear();
    let mut scope: BTreeMap<String, BindingId> = BTreeMap::new();
    let mut constants = HashMap::new();
    let mut bindings = Vec::with_capacity(program.bindings.len());
    let mut output = None;

    for declaration in &program.bindings {
        if let Some(first_id) = scope.get(&declaration.name.name).copied() {
            let first = symbols
                .get(first_id)
                .expect("a scoped binding must have a stored symbol");
            return Err(Diagnostic::error(
                ErrorCode::DuplicateBinding,
                format!("duplicate binding `{}`", declaration.name.name),
                DiagnosticLabel::new(
                    declaration.name.span,
                    "this name was already declared in the same scope",
                ),
            )
            .with_secondary(DiagnosticLabel::new(
                first.span,
                format!("first definition of `{}` is here", first.name),
            )));
        }

        let value = check_expression(&declaration.value, &scope, symbols, &constants)?;
        let value = apply_declaration_type(declaration, value, &constants)?;
        let constant = if value.ty == Type::Float {
            const_value(&value, &constants)?
        } else {
            None
        };
        let id = symbols.define(
            declaration.name.name.clone(),
            value.ty,
            declaration.name.span,
        );

        if let Some(constant) = constant {
            constants.insert(id, constant);
        }
        scope.insert(declaration.name.name.clone(), id);
        if declaration.name.name == "out" {
            output = Some(id);
        }
        bindings.push(TypedBinding {
            id,
            name: declaration.name.clone(),
            annotation: declaration.annotation.clone(),
            value,
            span: declaration.span,
        });
    }

    let output = output.ok_or_else(|| {
        Diagnostic::error(
            ErrorCode::MissingOutput,
            "missing required `out` binding",
            DiagnosticLabel::new(
                program.render.span,
                "add `let out = ...;` before this render declaration",
            ),
        )
    })?;
    let render = check_render(&program.render, sample_rate)?;

    Ok(TypedProgram {
        bindings,
        output,
        render,
        span: program.span,
    })
}

fn check_expression(
    expression: &Expr,
    scope: &BTreeMap<String, BindingId>,
    symbols: &SymbolStorage,
    constants: &HashMap<BindingId, f64>,
) -> Result<TypedExpr, Diagnostic> {
    match &expression.kind {
        ExprKind::Number(value) => {
            let expression = TypedExpr {
                kind: TypedExprKind::Number(*value),
                ty: Type::Float,
                span: expression.span,
            };
            if !value.is_finite() {
                return Err(invalid_float(expression.span));
            }
            Ok(expression)
        }
        ExprKind::Identifier(identifier) => {
            let id = scope.get(&identifier.name).copied().ok_or_else(|| {
                Diagnostic::error(
                    ErrorCode::UnknownName,
                    format!("unknown name `{}`", identifier.name),
                    DiagnosticLabel::new(
                        identifier.span,
                        format!("no previous binding defines `{}`", identifier.name),
                    ),
                )
                .with_note(
                    "declare bindings before use; self and forward references are not allowed",
                )
            })?;
            let symbol = symbols
                .get(id)
                .expect("a scoped binding must have a stored symbol");

            Ok(TypedExpr {
                kind: TypedExprKind::Identifier {
                    id,
                    identifier: identifier.clone(),
                },
                ty: symbol.ty,
                span: expression.span,
            })
        }
        ExprKind::Call { callee, arguments } => {
            let builtin = builtin_by_name(&callee.name).ok_or_else(|| {
                Diagnostic::error(
                    ErrorCode::UnknownFunction,
                    format!("unknown function `{}`", callee.name),
                    DiagnosticLabel::new(callee.span, "this callee is not a builtin function"),
                )
            })?;

            if arguments.len() != builtin.signature.parameters.len() {
                let expected = builtin.signature.parameters.len();
                let found = arguments.len();
                let expected_noun = if expected == 1 {
                    "argument"
                } else {
                    "arguments"
                };
                let found_noun = if found == 1 { "argument" } else { "arguments" };
                return Err(Diagnostic::error(
                    ErrorCode::InvalidArity,
                    format!("invalid argument count in call to `{}`", callee.name),
                    DiagnosticLabel::new(
                        callee.span,
                        format!("expected {expected} {expected_noun}, found {found} {found_noun}"),
                    ),
                ));
            }

            let mut typed_arguments = Vec::with_capacity(arguments.len());
            for (argument, expected) in arguments.iter().zip(builtin.signature.parameters.iter()) {
                let argument = check_expression(argument, scope, symbols, constants)?;
                typed_arguments.push(coerce_to_type(argument, *expected, None, constants)?);
            }

            Ok(TypedExpr {
                kind: TypedExprKind::Call {
                    builtin: builtin.id,
                    callee: callee.clone(),
                    arguments: typed_arguments,
                },
                ty: builtin.signature.result,
                span: expression.span,
            })
        }
        ExprKind::Unary { operator, operand } => {
            let operand = check_expression(operand, scope, symbols, constants)?;
            let ty = operand.ty;
            let result = TypedExpr {
                kind: TypedExprKind::Unary {
                    operator: *operator,
                    operand: Box::new(operand),
                },
                ty,
                span: expression.span,
            };
            if ty == Type::Float {
                const_value(&result, constants)?;
            }
            Ok(result)
        }
        ExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = check_expression(left, scope, symbols, constants)?;
            let right = check_expression(right, scope, symbols, constants)?;
            let ty = if left.ty == Type::Sig || right.ty == Type::Sig {
                Type::Sig
            } else {
                Type::Float
            };
            let left = if left.ty == Type::Float && ty == Type::Sig {
                lift_to_sig(left, constants)?
            } else {
                left
            };
            let right = if right.ty == Type::Float && ty == Type::Sig {
                lift_to_sig(right, constants)?
            } else {
                right
            };
            let result = TypedExpr {
                kind: TypedExprKind::Binary {
                    operator: *operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                ty,
                span: expression.span,
            };
            if ty == Type::Float {
                const_value(&result, constants)?;
            }
            Ok(result)
        }
        ExprKind::Group(inner) => {
            let inner = check_expression(inner, scope, symbols, constants)?;
            let ty = inner.ty;
            Ok(TypedExpr {
                kind: TypedExprKind::Group(Box::new(inner)),
                ty,
                span: expression.span,
            })
        }
    }
}

fn apply_declaration_type(
    declaration: &LetDecl,
    value: TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<TypedExpr, Diagnostic> {
    if declaration.name.name == "out" {
        if let Some(annotation) = &declaration.annotation
            && annotation.kind != TypeName::Sig
        {
            return Err(Diagnostic::error(
                ErrorCode::TypeMismatch,
                "`out` must have type `Sig`, not `Float`",
                DiagnosticLabel::new(
                    annotation.span,
                    "remove this annotation or change it to `Sig`",
                ),
            ));
        }
        return coerce_to_type(value, Type::Sig, None, constants);
    }

    if let Some(annotation) = &declaration.annotation {
        let expected = match annotation.kind {
            TypeName::Float => Type::Float,
            TypeName::Sig => Type::Sig,
        };
        return coerce_to_type(value, expected, Some(annotation.span), constants);
    }

    Ok(value)
}

fn coerce_to_type(
    expression: TypedExpr,
    expected: Type,
    secondary: Option<SourceSpan>,
    constants: &HashMap<BindingId, f64>,
) -> Result<TypedExpr, Diagnostic> {
    match (expression.ty, expected) {
        (actual, expected) if actual == expected => Ok(expression),
        (Type::Float, Type::Sig) => lift_to_sig(expression, constants),
        (actual, expected) => Err(type_mismatch(expression.span, expected, actual, secondary)),
    }
}

fn type_mismatch(
    span: SourceSpan,
    expected: Type,
    actual: Type,
    secondary: Option<SourceSpan>,
) -> Diagnostic {
    let label = if secondary.is_some() {
        format!("this value has type `{actual:?}`")
    } else {
        format!("this expression has type `{actual:?}`; expected `{expected:?}`")
    };
    let diagnostic = Diagnostic::error(
        ErrorCode::TypeMismatch,
        format!("type mismatch: expected `{expected:?}`, found `{actual:?}`"),
        DiagnosticLabel::new(span, label),
    );

    if let Some(span) = secondary {
        diagnostic.with_secondary(DiagnosticLabel::new(
            span,
            format!("this annotation requires type `{expected:?}`"),
        ))
    } else {
        diagnostic
    }
}

fn lift_to_sig(
    expression: TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<TypedExpr, Diagnostic> {
    let value =
        const_value(&expression, constants)?.ok_or_else(|| invalid_float(expression.span))?;
    let converted = value as f32;

    if !converted.is_finite() {
        return Err(invalid_lifted_float(expression.span));
    }

    Ok(TypedExpr {
        kind: TypedExprKind::LiftFloatToSig(Box::new(expression.clone())),
        ty: Type::Sig,
        span: expression.span,
    })
}

fn const_value(
    expression: &TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<Option<f64>, Diagnostic> {
    if expression.ty != Type::Float {
        return Ok(None);
    }

    let value = match &expression.kind {
        TypedExprKind::Number(value) => *value,
        TypedExprKind::Identifier { id, identifier } => {
            constants.get(id).copied().ok_or_else(|| {
                Diagnostic::error(
                    ErrorCode::UnknownName,
                    "unknown constant binding",
                    DiagnosticLabel::new(identifier.span, identifier.name.as_str()),
                )
            })?
        }
        TypedExprKind::Unary { operand, .. } => {
            let operand = required_const_value(operand, constants)?;
            -operand
        }
        TypedExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left_value = required_const_value(left, constants)?;
            let right_value = required_const_value(right, constants)?;
            if *operator == BinaryOp::Divide && right_value == 0.0 {
                return Err(Diagnostic::error(
                    ErrorCode::ConstantDivisionByZero,
                    "compile-time division by zero",
                    DiagnosticLabel::new(right.span, "this divisor evaluates to zero"),
                ));
            }

            match operator {
                BinaryOp::Add => left_value + right_value,
                BinaryOp::Subtract => left_value - right_value,
                BinaryOp::Multiply => left_value * right_value,
                BinaryOp::Divide => left_value / right_value,
            }
        }
        TypedExprKind::Group(inner) => required_const_value(inner, constants)?,
        TypedExprKind::Call { .. } | TypedExprKind::LiftFloatToSig(_) => return Ok(None),
    };

    if !value.is_finite() {
        return Err(invalid_float(expression.span));
    }

    Ok(Some(value))
}

fn required_const_value(
    expression: &TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<f64, Diagnostic> {
    const_value(expression, constants)?.ok_or_else(|| invalid_float(expression.span))
}

fn invalid_float(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::InvalidFloat,
        "compile-time `Float` must be finite",
        DiagnosticLabel::new(span, "this expression evaluates to NaN or infinity"),
    )
}

fn invalid_lifted_float(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::InvalidFloat,
        "`Float` value is too large for `Sig`",
        DiagnosticLabel::new(
            span,
            "converting this expression to `f32` produces infinity",
        ),
    )
}

fn check_render(render: &RenderDecl, sample_rate: u32) -> Result<TypedRender, Diagnostic> {
    let mut duration_seconds = match render.duration.unit {
        crate::syntax::DurationUnit::Seconds => render.duration.value,
        crate::syntax::DurationUnit::Milliseconds => render.duration.value / 1_000.0,
    };
    if render.negative {
        duration_seconds = -duration_seconds;
    }

    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err(invalid_duration(render, sample_rate));
    }

    let scaled = duration_seconds * f64::from(sample_rate);
    if !scaled.is_finite() {
        return Err(render_overflow(render));
    }

    let rounded = scaled.round();
    if rounded == 0.0 {
        return Err(invalid_duration(render, sample_rate));
    }

    const MAX_WAV_SAMPLES: u64 = (u32::MAX as u64 - 36) / 2;
    if rounded >= 18_446_744_073_709_551_616.0 || rounded > MAX_WAV_SAMPLES as f64 {
        return Err(render_overflow(render));
    }

    Ok(TypedRender {
        duration: render.duration.clone(),
        duration_seconds,
        render_samples: rounded as u64,
        span: render.span,
    })
}

fn invalid_duration(render: &RenderDecl, sample_rate: u32) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::InvalidDuration,
        "invalid render duration",
        DiagnosticLabel::new(
            render.duration.span,
            format!(
                "expected a finite, positive duration that rounds to at least one sample at {sample_rate} Hz"
            ),
        ),
    )
}

fn render_overflow(render: &RenderDecl) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::RenderOverflow,
        "render duration exceeds WAV size limits",
        DiagnosticLabel::new(
            render.duration.span,
            "sample count or RIFF/PCM size overflows its supported integer range",
        ),
    )
}
