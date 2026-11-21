use std::collections::BTreeMap;

use crate::{
    SourceSpan,
    ir::{CallOrigin, Node, NodeId, NodeOp, NodeOrigin, ValueType},
    sema::{
        BindingId, BuiltinId, FunctionId, Type, TypedBinding, TypedExpr, TypedFunction,
        TypedProgram, TypedTopDecl,
    },
    syntax::BinaryOp,
};

use super::Graph;

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

fn lower_expr(value: &TypedExpr, program: &TypedProgram, lowering: &mut Lowering) -> NodeId {
    let ty = match value.ty {
        Type::Float => ValueType::Float,
        Type::Sig => ValueType::Sig,
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
        } => {
            assert_eq!(
                *builtin,
                BuiltinId::SINE,
                "only resolved sine calls can reach lowering"
            );
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
