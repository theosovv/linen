use std::collections::BTreeMap;

use crate::{
    SourceSpan,
    ir::{CallOrigin, Node, NodeId, NodeOp, NodeOrigin, RandomStreamId, ValueType},
    sema::{
        BindingId, BuiltinId, FunctionId, Type, TypedBinding, TypedExpr, TypedFunction,
        TypedProgram, TypedTopDecl,
    },
    syntax::BinaryOp,
};

use super::{Graph, graph::const_float_value};

struct Lowering<'a> {
    nodes: Vec<Node>,
    roots: Vec<NodeId>,
    bindings: BTreeMap<BindingId, NodeId>,
    functions: Vec<&'a TypedFunction>,
    call_stack: Vec<CallOrigin>,
}

fn push_node(
    lowering: &mut Lowering,
    op: NodeOp,
    result_type: ValueType,
    primary: SourceSpan,
) -> NodeId {
    let id = NodeId::new(lowering.nodes.len());
    let origin = NodeOrigin {
        primary,
        inlined_from: lowering.call_stack.iter().rev().copied().collect(),
    };

    lowering.nodes.push(Node {
        id,
        op,
        result_type,
        origin,
    });

    id
}

fn restore_bindings(lowering: &mut Lowering, saved: Vec<(BindingId, Option<NodeId>)>) {
    for (id, previous) in saved.into_iter().rev() {
        if let Some(previous) = previous {
            lowering.bindings.insert(id, previous);
        } else {
            lowering.bindings.remove(&id);
        }
    }
}

fn lower_block(
    bindings: &[TypedBinding],
    result: &TypedExpr,
    program: &TypedProgram,
    lowering: &mut Lowering,
) -> NodeId {
    let mut saved = Vec::with_capacity(bindings.len());

    for binding in bindings {
        let value = lower_expr(&binding.value, program, lowering);
        let previous = lowering.bindings.insert(binding.id, value);
        saved.push((binding.id, previous));
        lowering.roots.push(value);
    }

    let result = lower_expr(result, program, lowering);
    restore_bindings(lowering, saved);
    result
}

fn lower_function_call(
    expression: &TypedExpr,
    function: FunctionId,
    arguments: &[TypedExpr],
    program: &TypedProgram,
    lowering: &mut Lowering,
) -> NodeId {
    let definition = *lowering
        .functions
        .get(function.index())
        .expect("a resolved function must have a typed definition");
    let mut saved = Vec::with_capacity(arguments.len());

    for (argument, parameter) in arguments.iter().zip(&definition.parameters) {
        let value = lower_expr(argument, program, lowering);
        lowering.roots.push(value);
        let previous = lowering.bindings.insert(parameter.id, value);
        saved.push((parameter.id, previous));
    }

    let call = CallOrigin {
        function,
        call_span: expression.span,
    };
    lowering.call_stack.push(call);
    let result = lower_expr(&definition.body, program, lowering);
    let popped = lowering.call_stack.pop();
    debug_assert_eq!(popped, Some(call));

    restore_bindings(lowering, saved);
    result
}

fn lower_builtin(
    value: &TypedExpr,
    builtin: BuiltinId,
    arguments: &[TypedExpr],
    program: &TypedProgram,
    lowering: &mut Lowering,
) -> NodeId {
    match builtin {
        BuiltinId::SINE => {
            assert_eq!(
                arguments.len(),
                1,
                "sine arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Sine(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        BuiltinId::PHASOR => {
            assert_eq!(
                arguments.len(),
                1,
                "phasor arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Phasor(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        BuiltinId::SAW => {
            assert_eq!(
                arguments.len(),
                1,
                "saw arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Saw(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        BuiltinId::SQUARE => {
            assert_eq!(
                arguments.len(),
                1,
                "square arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Square(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        BuiltinId::TRIANGLE => {
            assert_eq!(
                arguments.len(),
                1,
                "triangle arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Triangle(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        BuiltinId::GAIN => {
            assert_eq!(
                arguments.len(),
                2,
                "gain arity must be checked before lowering"
            );

            let input = lower_expr(&arguments[0], program, lowering);
            let amount = lower_expr(&arguments[1], program, lowering);

            push_node(
                lowering,
                NodeOp::Mul(input, amount),
                ValueType::Sig,
                value.span,
            )
        }
        BuiltinId::MIX => {
            assert_eq!(
                arguments.len(),
                3,
                "mix arity must be checked before lowering"
            );

            let a = lower_expr(&arguments[0], program, lowering);
            let b = lower_expr(&arguments[1], program, lowering);
            let balance = lower_expr(&arguments[2], program, lowering);

            let one = push_node(
                lowering,
                NodeOp::ConstFloat(1.0),
                ValueType::Float,
                value.span,
            );
            let one = push_node(
                lowering,
                NodeOp::LiftFloatToSig(one),
                ValueType::Sig,
                value.span,
            );
            let a_weight = push_node(
                lowering,
                NodeOp::Sub(one, balance),
                ValueType::Sig,
                value.span,
            );
            let left = push_node(
                lowering,
                NodeOp::Mul(a, a_weight),
                ValueType::Sig,
                value.span,
            );
            let right = push_node(
                lowering,
                NodeOp::Mul(b, balance),
                ValueType::Sig,
                value.span,
            );

            push_node(
                lowering,
                NodeOp::Add(left, right),
                ValueType::Sig,
                value.span,
            )
        }
        BuiltinId::STEREO => {
            assert_eq!(
                arguments.len(),
                2,
                "stereo arity must be checked before lowering"
            );

            let left = lower_expr(&arguments[0], program, lowering);
            let right = lower_expr(&arguments[1], program, lowering);

            push_node(
                lowering,
                NodeOp::Stereo(left, right),
                ValueType::Stereo,
                value.span,
            )
        }
        BuiltinId::PAN => {
            assert_eq!(
                arguments.len(),
                2,
                "pan arity must be checked before lowering"
            );

            let input = lower_expr(&arguments[0], program, lowering);
            let position_expression = &arguments[1];
            let position = lower_expr(position_expression, program, lowering);

            push_node(
                lowering,
                NodeOp::Pan(input, position),
                ValueType::Stereo,
                position_expression.span,
            )
        }
        BuiltinId::NOISE => {
            assert_eq!(
                arguments.len(),
                1,
                "noise arity must be checked before lowering"
            );

            let seed = lower_expr(&arguments[0], program, lowering);
            let seed_value = const_float_value(&lowering.nodes, seed)
                .expect("noise seed must be a checked compile-time Float");

            push_node(
                lowering,
                NodeOp::Noise {
                    seed,
                    stream: RandomStreamId::new(seed_value as u64),
                },
                ValueType::Sig,
                value.span,
            )
        }
        _ => panic!("unknown builtin ID {}", builtin.index()),
    }
}

fn lower_expr(value: &TypedExpr, program: &TypedProgram, lowering: &mut Lowering) -> NodeId {
    let ty = match value.ty {
        Type::Float => ValueType::Float,
        Type::Sig => ValueType::Sig,
        Type::Stereo => ValueType::Stereo,
    };

    match &value.kind {
        crate::sema::TypedExprKind::Number(val) => push_node(
            lowering,
            NodeOp::ConstFloat(*val),
            ValueType::Float,
            value.span,
        ),
        crate::sema::TypedExprKind::Identifier { id, identifier: _ } => lowering
            .bindings
            .get(id)
            .copied()
            .expect("resolved binding must be lowered before use"),
        crate::sema::TypedExprKind::BuiltinCall {
            builtin,
            callee: _,
            arguments,
        } => lower_builtin(value, *builtin, arguments, program, lowering),
        crate::sema::TypedExprKind::FunctionCall {
            function,
            callee: _,
            arguments,
        } => lower_function_call(value, *function, arguments, program, lowering),
        crate::sema::TypedExprKind::Unary {
            operator: _,
            operand,
        } => {
            let operand = lower_expr(operand, program, lowering);
            push_node(lowering, NodeOp::Neg(operand), ty, value.span)
        }
        crate::sema::TypedExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = lower_expr(left, program, lowering);
            let right = lower_expr(right, program, lowering);
            let op = match operator {
                BinaryOp::Add => NodeOp::Add(left, right),
                BinaryOp::Subtract => NodeOp::Sub(left, right),
                BinaryOp::Multiply => NodeOp::Mul(left, right),
                BinaryOp::Divide => NodeOp::Div(left, right),
            };

            push_node(lowering, op, ty, value.span)
        }
        crate::sema::TypedExprKind::Group(typed_expr) => lower_expr(typed_expr, program, lowering),
        crate::sema::TypedExprKind::Block { bindings, result } => {
            lower_block(bindings, result, program, lowering)
        }
        crate::sema::TypedExprKind::LiftFloatToSig(typed_expr) => {
            let inner = lower_expr(typed_expr, program, lowering);
            push_node(
                lowering,
                NodeOp::LiftFloatToSig(inner),
                ValueType::Sig,
                typed_expr.span,
            )
        }
    }
}

pub fn lower(program: &TypedProgram) -> Graph {
    let functions = program.functions().collect();
    let mut lowering = Lowering {
        nodes: Vec::new(),
        roots: Vec::new(),
        bindings: BTreeMap::new(),
        functions,
        call_stack: Vec::new(),
    };

    for declaration in &program.declarations {
        let TypedTopDecl::Binding(binding) = declaration else {
            continue;
        };
        let value = lower_expr(&binding.value, program, &mut lowering);

        lowering.bindings.insert(binding.id, value);
        lowering.roots.push(value);
    }

    let output = lowering
        .bindings
        .get(&program.output)
        .copied()
        .expect("typed output must already be lowered");

    Graph {
        nodes: lowering.nodes,
        roots: lowering.roots,
        output,
        render_samples: program.render.render_samples,
    }
}
