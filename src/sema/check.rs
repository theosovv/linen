use std::collections::{BTreeMap, HashMap};

use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode,
    source::SourceSpan,
    syntax::{
        BinaryOp, Expr, ExprKind, FunctionDecl, Identifier, LetDecl, Program, RenderDecl, TopDecl,
        TypeName,
    },
};

use super::{
    BindingId, FunctionLimits, Scopes, SymbolStorage, Type, TypedBinding, TypedExpr, TypedExprKind,
    TypedFunction, TypedParameter, TypedProgram, TypedRender, TypedTopDecl, builtin_by_name,
};

fn duplicate_declaration(name: &Identifier, first_span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::DuplicateBinding,
        format!("duplicate binding `{}`", name.name),
        DiagnosticLabel::new(
            name.span,
            "this name was already declared in the same scope",
        ),
    )
    .with_secondary(DiagnosticLabel::new(
        first_span,
        format!("first definition of `{}` is here", name.name),
    ))
}

pub fn check_program(
    program: &Program,
    symbols: &mut SymbolStorage,
    sample_rate: u32,
    limits: FunctionLimits,
) -> Result<TypedProgram, Diagnostic> {
    symbols.clear();
    let mut scopes = Scopes::new();
    let mut constants = HashMap::new();
    let mut declarations = Vec::with_capacity(program.declarations.len());
    let mut declared_names: BTreeMap<String, SourceSpan> = BTreeMap::new();
    let mut functions = Vec::new();
    let mut evaluator = ExpansionEvaluator::new(limits);
    let mut output = None;

    for top_declaration in &program.declarations {
        match top_declaration {
            TopDecl::Function(function) => {
                if let Some(first_span) = declared_names.get(&function.name.name).copied() {
                    return Err(duplicate_declaration(&function.name, first_span));
                }

                if function.name.name == "sine" {
                    return Err(Diagnostic::error(
                        ErrorCode::DuplicateBinding,
                        "function `sine` conflicts with a builtin",
                        DiagnosticLabel::new(
                            function.name.span,
                            "`sine` is already defined as a builtin function",
                        ),
                    )
                    .with_note("user-defined functions cannot replace builtins"));
                }

                let typed = check_function(
                    function,
                    &mut scopes,
                    symbols,
                    &constants,
                    &functions,
                    &mut evaluator,
                )?;

                scopes.define_function(function.name.name.clone(), typed.id);
                declared_names.insert(function.name.name.clone(), function.name.span);
                functions.push(typed.clone());
                declarations.push(TypedTopDecl::Function(typed));
            }
            TopDecl::Let(declaration) => {
                if let Some(first_span) = declared_names.get(&declaration.name.name).copied() {
                    return Err(duplicate_declaration(&declaration.name, first_span));
                }

                let value = check_expression(
                    &declaration.value,
                    &mut scopes,
                    symbols,
                    &constants,
                    &functions,
                    &mut evaluator,
                )?;
                let value = apply_declaration_type(declaration, value, &constants)?;
                let constant =
                    evaluator.evaluate(&value, &constants, &functions, &mut HashMap::new())?;
                if value.ty == Type::Float && constant.is_none() {
                    return Err(invalid_float(value.span));
                }

                let id = symbols.define(
                    declaration.name.name.clone(),
                    value.ty,
                    declaration.name.span,
                );

                if let Some(constant) = constant {
                    constants.insert(id, constant);
                }

                scopes.define_value(declaration.name.name.clone(), id);
                declared_names.insert(declaration.name.name.clone(), declaration.name.span);

                if declaration.name.name == "out" {
                    output = Some(id);
                }

                declarations.push(TypedTopDecl::Binding(TypedBinding {
                    id,
                    name: declaration.name.clone(),
                    annotation: declaration.annotation.clone(),
                    value,
                    span: declaration.span,
                }));
            }
        }
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
        declarations,
        output,
        render,
        span: program.span,
    })
}

fn check_function(
    function: &FunctionDecl,
    scopes: &mut Scopes,
    symbols: &mut SymbolStorage,
    constants: &HashMap<BindingId, f64>,
    functions: &[TypedFunction],
    evaluator: &mut ExpansionEvaluator,
) -> Result<TypedFunction, Diagnostic> {
    scopes.push_scope();

    let result = (|| {
        let mut parameters = Vec::with_capacity(function.parameters.len());
        let mut parameter_types = Vec::with_capacity(function.parameters.len());

        for parameter in &function.parameters {
            if let Some(first_id) = scopes.current_value(&parameter.name.name) {
                let first = symbols
                    .get(first_id)
                    .expect("a parameter binding must have a stored symbol");

                return Err(duplicate_declaration(&parameter.name, first.span));
            }

            let ty = match parameter.annotation.kind {
                TypeName::Float => Type::Float,
                TypeName::Sig => Type::Sig,
            };
            let id = symbols.define(parameter.name.name.clone(), ty, parameter.name.span);

            scopes.define_value(parameter.name.name.clone(), id);
            parameter_types.push(ty);
            parameters.push(TypedParameter {
                id,
                name: parameter.name.clone(),
                annotation: parameter.annotation.clone(),
                span: parameter.span,
            });
        }

        let body = {
            let mut context = CheckContext {
                scopes,
                symbols,
                constants,
                functions,
                evaluator,
            };
            match &function.body.kind {
                ExprKind::Block { bindings, result } => {
                    check_block_contents(&function.body, bindings, result, &mut context)?
                }
                _ => check_expression(
                    &function.body,
                    context.scopes,
                    context.symbols,
                    context.constants,
                    context.functions,
                    context.evaluator,
                )?,
            }
        };
        let result_type = match function.result.kind {
            TypeName::Float => Type::Float,
            TypeName::Sig => Type::Sig,
        };
        let body = coerce_to_type(body, result_type, Some(function.result.span), constants)?;

        let mut generic_values = HashMap::new();
        for parameter in &parameters {
            if parameter.annotation.kind == TypeName::Float {
                generic_values.insert(parameter.id, None);
            }
        }
        evaluator.evaluate(&body, constants, functions, &mut generic_values)?;

        let parameter_spans = parameters
            .iter()
            .map(|parameter| parameter.annotation.span)
            .collect();
        let id = symbols.define_function(
            function.name.name.clone(),
            parameter_types,
            parameter_spans,
            body.ty,
            function.name.span,
        );

        Ok(TypedFunction {
            id,
            name: function.name.clone(),
            parameters,
            result: function.result.clone(),
            body,
            span: function.span,
        })
    })();

    scopes.pop_scope();
    result
}

struct CheckContext<'a> {
    scopes: &'a mut Scopes,
    symbols: &'a mut SymbolStorage,
    constants: &'a HashMap<BindingId, f64>,
    functions: &'a [TypedFunction],
    evaluator: &'a mut ExpansionEvaluator,
}

fn check_block(
    expression: &Expr,
    bindings: &[LetDecl],
    result: &Expr,
    mut context: CheckContext<'_>,
) -> Result<TypedExpr, Diagnostic> {
    context.scopes.push_scope();
    let result = check_block_contents(expression, bindings, result, &mut context);
    context.scopes.pop_scope();
    result
}

fn check_block_contents(
    expression: &Expr,
    bindings: &[LetDecl],
    result: &Expr,
    context: &mut CheckContext<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let mut typed_bindings = Vec::with_capacity(bindings.len());

    for binding in bindings {
        if let Some(first_id) = context.scopes.current_value(&binding.name.name) {
            let first = context
                .symbols
                .get(first_id)
                .expect("a scoped binding must have a stored symbol");

            return Err(duplicate_declaration(&binding.name, first.span));
        }

        let value = check_expression(
            &binding.value,
            context.scopes,
            context.symbols,
            context.constants,
            context.functions,
            context.evaluator,
        )?;
        let value = apply_local_annotation(binding, value, context.constants)?;
        let id = context
            .symbols
            .define(binding.name.name.clone(), value.ty, binding.name.span);

        context.scopes.define_value(binding.name.name.clone(), id);
        typed_bindings.push(TypedBinding {
            id,
            name: binding.name.clone(),
            annotation: binding.annotation.clone(),
            value,
            span: binding.span,
        });
    }

    let typed_result = check_expression(
        result,
        context.scopes,
        context.symbols,
        context.constants,
        context.functions,
        context.evaluator,
    )?;
    let ty = typed_result.ty;

    Ok(TypedExpr {
        kind: TypedExprKind::Block {
            bindings: typed_bindings,
            result: Box::new(typed_result),
        },
        ty,
        span: expression.span,
    })
}

fn apply_local_annotation(
    binding: &LetDecl,
    value: TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<TypedExpr, Diagnostic> {
    let Some(annotation) = &binding.annotation else {
        return Ok(value);
    };
    let expected = match annotation.kind {
        TypeName::Float => Type::Float,
        TypeName::Sig => Type::Sig,
    };

    coerce_to_type(value, expected, Some(annotation.span), constants)
}

fn check_function_call(
    expression: &Expr,
    callee: &Identifier,
    arguments: &[Expr],
    context: &mut CheckContext<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let function = context
        .scopes
        .lookup_function(&callee.name)
        .ok_or_else(|| {
            Diagnostic::error(
                ErrorCode::UnknownFunction,
                format!("unknown function `{}`", callee.name),
                DiagnosticLabel::new(callee.span, "this callee is not a builtin function"),
            )
        })?;
    let definition = context
        .symbols
        .get_function(function)
        .expect("a resolved function must have a stored symbol")
        .clone();
    let expected = definition.parameter_types.len();
    let found = arguments.len();

    if found != expected {
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
    for (argument, (expected_type, parameter_span)) in arguments.iter().zip(
        definition
            .parameter_types
            .iter()
            .zip(&definition.parameter_spans),
    ) {
        let argument = check_expression(
            argument,
            context.scopes,
            context.symbols,
            context.constants,
            context.functions,
            context.evaluator,
        )?;
        typed_arguments.push(coerce_to_type(
            argument,
            *expected_type,
            Some(*parameter_span),
            context.constants,
        )?);
    }

    Ok(TypedExpr {
        kind: TypedExprKind::FunctionCall {
            function,
            callee: callee.clone(),
            arguments: typed_arguments,
        },
        ty: definition.result,
        span: expression.span,
    })
}

fn check_expression(
    expression: &Expr,
    scopes: &mut Scopes,
    symbols: &mut SymbolStorage,
    constants: &HashMap<BindingId, f64>,
    functions: &[TypedFunction],
    evaluator: &mut ExpansionEvaluator,
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
            let id = scopes.lookup_value(&identifier.name).ok_or_else(|| {
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
            let Some(builtin) = builtin_by_name(&callee.name) else {
                if scopes.lookup_function(&callee.name).is_some() {
                    let mut context = CheckContext {
                        scopes,
                        symbols,
                        constants,
                        functions,
                        evaluator,
                    };
                    return check_function_call(expression, callee, arguments, &mut context);
                }
                return Err(Diagnostic::error(
                    ErrorCode::UnknownFunction,
                    format!("unknown function `{}`", callee.name),
                    DiagnosticLabel::new(callee.span, "this callee is not a builtin function"),
                ));
            };

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
                let argument =
                    check_expression(argument, scopes, symbols, constants, functions, evaluator)?;
                typed_arguments.push(coerce_to_type(argument, *expected, None, constants)?);
            }

            Ok(TypedExpr {
                kind: TypedExprKind::BuiltinCall {
                    builtin: builtin.id,
                    callee: callee.clone(),
                    arguments: typed_arguments,
                },
                ty: builtin.signature.result,
                span: expression.span,
            })
        }
        ExprKind::Unary { operator, operand } => {
            let operand =
                check_expression(operand, scopes, symbols, constants, functions, evaluator)?;
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
                let _ = evaluate_known(&result, constants)?;
            }
            Ok(result)
        }
        ExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = check_expression(left, scopes, symbols, constants, functions, evaluator)?;
            let right = check_expression(right, scopes, symbols, constants, functions, evaluator)?;
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
                let _ = evaluate_known(&result, constants)?;
            }
            Ok(result)
        }
        ExprKind::Group(inner) => {
            let inner = check_expression(inner, scopes, symbols, constants, functions, evaluator)?;
            let ty = inner.ty;
            Ok(TypedExpr {
                kind: TypedExprKind::Group(Box::new(inner)),
                ty,
                span: expression.span,
            })
        }
        ExprKind::Block { bindings, result } => {
            let context = CheckContext {
                scopes,
                symbols,
                constants,
                functions,
                evaluator,
            };
            check_block(expression, bindings, result, context)
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

    apply_local_annotation(declaration, value, constants)
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
    if let Some(value) = evaluate_known(&expression, constants)? {
        let converted = value as f32;
        if !converted.is_finite() {
            return Err(invalid_lifted_float(expression.span));
        }
    }

    let span = expression.span;
    Ok(TypedExpr {
        kind: TypedExprKind::LiftFloatToSig(Box::new(expression)),
        ty: Type::Sig,
        span,
    })
}

fn evaluate_known(
    expression: &TypedExpr,
    constants: &HashMap<BindingId, f64>,
) -> Result<Option<f64>, Diagnostic> {
    let mut evaluator = ExpansionEvaluator::new(FunctionLimits::default());
    evaluator.known_only = true;
    evaluator.evaluate(expression, constants, &[], &mut HashMap::new())
}

struct ExpansionEvaluator {
    limits: FunctionLimits,
    call_depth: usize,
    expanded_nodes: usize,
    expansion_root: Option<SourceSpan>,
    call_stack: Vec<SourceSpan>,
    known_only: bool,
}

impl ExpansionEvaluator {
    fn new(limits: FunctionLimits) -> Self {
        Self {
            limits,
            call_depth: 0,
            expanded_nodes: 0,
            expansion_root: None,
            call_stack: Vec::new(),
            known_only: false,
        }
    }

    fn evaluate(
        &mut self,
        expression: &TypedExpr,
        constants: &HashMap<BindingId, f64>,
        functions: &[TypedFunction],
        values: &mut HashMap<BindingId, Option<f64>>,
    ) -> Result<Option<f64>, Diagnostic> {
        self.evaluate_expression(expression, constants, functions, values)
    }

    fn evaluate_expression(
        &mut self,
        expression: &TypedExpr,
        constants: &HashMap<BindingId, f64>,
        functions: &[TypedFunction],
        values: &mut HashMap<BindingId, Option<f64>>,
    ) -> Result<Option<f64>, Diagnostic> {
        match &expression.kind {
            TypedExprKind::Number(value) => {
                self.charge_node()?;
                if !value.is_finite() {
                    return Err(
                        self.with_call_chain(invalid_float(expression.span), expression.span)
                    );
                }
                Ok(Some(*value))
            }
            TypedExprKind::Identifier { id, identifier } => {
                self.charge_node()?;
                if expression.ty != Type::Float {
                    return Ok(None);
                }
                if let Some(value) = values.get(id) {
                    return Ok(*value);
                }
                if let Some(value) = constants.get(id) {
                    return Ok(Some(*value));
                }
                if self.known_only {
                    return Ok(None);
                }

                let diagnostic = Diagnostic::error(
                    ErrorCode::UnknownName,
                    "unknown constant binding",
                    DiagnosticLabel::new(identifier.span, identifier.name.as_str()),
                );
                Err(self.with_call_chain(diagnostic, expression.span))
            }
            TypedExprKind::BuiltinCall { arguments, .. } => {
                self.charge_node()?;
                for argument in arguments {
                    self.evaluate_expression(argument, constants, functions, values)?;
                }
                Ok(None)
            }
            TypedExprKind::FunctionCall {
                function,
                arguments,
                ..
            } => {
                if self.known_only {
                    return Ok(None);
                }
                self.evaluate_function_call(
                    expression, *function, arguments, constants, functions, values,
                )
            }
            TypedExprKind::Unary { operand, .. } => {
                self.charge_node()?;
                let operand = self.evaluate_expression(operand, constants, functions, values)?;
                if expression.ty != Type::Float {
                    return Ok(None);
                }
                let Some(operand) = operand else {
                    return Ok(None);
                };
                let value = -operand;
                if !value.is_finite() {
                    return Err(
                        self.with_call_chain(invalid_float(expression.span), expression.span)
                    );
                }
                Ok(Some(value))
            }
            TypedExprKind::Binary {
                operator,
                left,
                right,
            } => {
                self.charge_node()?;
                let left = self.evaluate_expression(left, constants, functions, values)?;
                let right_source_span = right.span;
                let right = self.evaluate_expression(right, constants, functions, values)?;
                if expression.ty != Type::Float {
                    return Ok(None);
                }
                if *operator == BinaryOp::Divide && right == Some(0.0) {
                    let diagnostic = Diagnostic::error(
                        ErrorCode::ConstantDivisionByZero,
                        "compile-time division by zero",
                        DiagnosticLabel::new(right_source_span, "this divisor evaluates to zero"),
                    );
                    return Err(self.with_call_chain(diagnostic, right_source_span));
                }
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                let value = match operator {
                    BinaryOp::Add => left + right,
                    BinaryOp::Subtract => left - right,
                    BinaryOp::Multiply => left * right,
                    BinaryOp::Divide => left / right,
                };
                if !value.is_finite() {
                    return Err(
                        self.with_call_chain(invalid_float(expression.span), expression.span)
                    );
                }
                Ok(Some(value))
            }
            TypedExprKind::Group(inner) => {
                self.charge_node()?;
                self.evaluate_expression(inner, constants, functions, values)
            }
            TypedExprKind::Block { bindings, result } => {
                self.charge_node()?;
                self.evaluate_block(bindings, result, constants, functions, values)
            }
            TypedExprKind::LiftFloatToSig(inner) => {
                self.charge_node()?;
                let value = self.evaluate_expression(inner, constants, functions, values)?;
                if let Some(value) = value {
                    let converted = value as f32;
                    if !converted.is_finite() {
                        let diagnostic = invalid_lifted_float(expression.span);
                        return Err(self.with_call_chain(diagnostic, expression.span));
                    }
                }
                Ok(None)
            }
        }
    }

    fn evaluate_block(
        &mut self,
        bindings: &[TypedBinding],
        result: &TypedExpr,
        constants: &HashMap<BindingId, f64>,
        functions: &[TypedFunction],
        values: &mut HashMap<BindingId, Option<f64>>,
    ) -> Result<Option<f64>, Diagnostic> {
        let mut saved = Vec::with_capacity(bindings.len());
        let result = (|| {
            for binding in bindings {
                let value =
                    self.evaluate_expression(&binding.value, constants, functions, values)?;
                saved.push((binding.id, values.get(&binding.id).copied()));
                values.insert(binding.id, value);
            }
            self.evaluate_expression(result, constants, functions, values)
        })();

        for (id, previous) in saved.into_iter().rev() {
            if let Some(previous) = previous {
                values.insert(id, previous);
            } else {
                values.remove(&id);
            }
        }
        result
    }

    fn evaluate_function_call(
        &mut self,
        expression: &TypedExpr,
        function: super::FunctionId,
        arguments: &[TypedExpr],
        constants: &HashMap<BindingId, f64>,
        functions: &[TypedFunction],
        values: &mut HashMap<BindingId, Option<f64>>,
    ) -> Result<Option<f64>, Diagnostic> {
        if self.call_depth >= self.limits.max_call_depth {
            let diagnostic = Diagnostic::error(
                ErrorCode::FunctionExpansionLimit,
                "function call depth limit exceeded",
                DiagnosticLabel::new(
                    expression.span,
                    format!(
                        "this call exceeds the configured maximum depth of {}",
                        self.limits.max_call_depth
                    ),
                ),
            );
            return Err(self.with_call_chain(diagnostic, expression.span));
        }

        let is_root = self.expansion_root.is_none();
        if is_root {
            self.expansion_root = Some(expression.span);
        }
        self.call_depth += 1;

        let definition = functions
            .get(function.index())
            .expect("a resolved function must have a typed definition");
        let mut saved = Vec::with_capacity(arguments.len());
        let result = (|| {
            for (argument, parameter) in arguments.iter().zip(&definition.parameters) {
                let value = self.evaluate_expression(argument, constants, functions, values)?;
                saved.push((parameter.id, values.get(&parameter.id).copied()));
                values.insert(parameter.id, value);
            }

            self.call_stack.push(expression.span);
            let result = self.evaluate_expression(&definition.body, constants, functions, values);
            self.call_stack.pop();
            result
        })();

        for (id, previous) in saved.into_iter().rev() {
            if let Some(previous) = previous {
                values.insert(id, previous);
            } else {
                values.remove(&id);
            }
        }
        self.call_depth -= 1;
        if is_root {
            self.expansion_root = None;
        }
        result
    }

    fn charge_node(&mut self) -> Result<(), Diagnostic> {
        if self.call_depth == 0 {
            return Ok(());
        }
        if self.expanded_nodes >= self.limits.max_expanded_nodes {
            let primary = self
                .expansion_root
                .expect("an active expansion must have a root call");
            let diagnostic = Diagnostic::error(
                ErrorCode::FunctionExpansionLimit,
                "function expansion node limit exceeded",
                DiagnosticLabel::new(
                    primary,
                    format!(
                        "this call exceeds the configured maximum of {} expanded expression visits",
                        self.limits.max_expanded_nodes
                    ),
                ),
            );
            return Err(self.with_call_chain(diagnostic, primary));
        }

        self.expanded_nodes += 1;
        Ok(())
    }

    fn with_call_chain(&self, mut diagnostic: Diagnostic, primary: SourceSpan) -> Diagnostic {
        for &call_span in self.call_stack.iter().rev() {
            if call_span == primary {
                continue;
            }
            diagnostic = diagnostic.with_secondary(DiagnosticLabel::new(
                call_span,
                "inlined from this function call",
            ));
        }
        diagnostic
    }
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
