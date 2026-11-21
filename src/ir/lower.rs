use crate::{
    SourceSpan,
    ir::{Node, NodeId, NodeOp, NodeOrigin, ValueType},
    sema::{BuiltinId, Type, TypedExpr, TypedProgram},
    syntax::BinaryOp,
};

use super::Graph;

struct Lowering {
    nodes: Vec<Node>,
    roots: Vec<NodeId>,
    bindings: Vec<Option<NodeId>>,
}

fn push_node(
    nodes: &mut Vec<Node>,
    op: NodeOp,
    result_type: ValueType,
    span: SourceSpan,
) -> NodeId {
    let id = NodeId::new(nodes.len());

    nodes.push(Node {
        id,
        op,
        result_type,
        origin: NodeOrigin { primary: span },
    });

    id
}

fn lower_expr(value: &TypedExpr, lowering: &mut Lowering) -> NodeId {
    let ty = match value.ty {
        Type::Float => ValueType::Float,
        Type::Sig => ValueType::Sig,
    };

    match &value.kind {
        crate::sema::TypedExprKind::Number(val) => push_node(
            &mut lowering.nodes,
            NodeOp::ConstFloat(*val),
            ValueType::Float,
            value.span,
        ),
        crate::sema::TypedExprKind::Identifier { id, identifier: _ } => lowering
            .bindings
            .get(id.index())
            .copied()
            .flatten()
            .expect("resolved binding must be lowered before use"),
        crate::sema::TypedExprKind::Call {
            builtin,
            callee: _,
            arguments,
        } => {
            assert_eq!(
                *builtin,
                BuiltinId::SINE,
                "only resolved sine calls can reach stage 3 lowering"
            );
            assert_eq!(
                arguments.len(),
                1,
                "sine arity must be checked before lowering"
            );

            let frequency_expression = &arguments[0];
            let frequency = lower_expr(frequency_expression, lowering);

            push_node(
                &mut lowering.nodes,
                NodeOp::Sine(frequency),
                ValueType::Sig,
                frequency_expression.span,
            )
        }
        crate::sema::TypedExprKind::Unary {
            operator: _,
            operand,
        } => {
            let operand = lower_expr(operand, lowering);
            push_node(&mut lowering.nodes, NodeOp::Neg(operand), ty, value.span)
        }
        crate::sema::TypedExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = lower_expr(left, lowering);
            let right = lower_expr(right, lowering);
            let op = match operator {
                BinaryOp::Add => NodeOp::Add(left, right),
                BinaryOp::Subtract => NodeOp::Sub(left, right),
                BinaryOp::Multiply => NodeOp::Mul(left, right),
                BinaryOp::Divide => NodeOp::Div(left, right),
            };

            push_node(&mut lowering.nodes, op, ty, value.span)
        }
        crate::sema::TypedExprKind::Group(typed_expr) => lower_expr(typed_expr, lowering),
        crate::sema::TypedExprKind::LiftFloatToSig(typed_expr) => {
            let inner = lower_expr(typed_expr, lowering);
            push_node(
                &mut lowering.nodes,
                NodeOp::LiftFloatToSig(inner),
                ValueType::Sig,
                typed_expr.span,
            )
        }
    }
}

pub fn lower(program: &TypedProgram) -> Graph {
    let mut lowering = Lowering {
        nodes: Vec::new(),
        roots: Vec::new(),
        bindings: vec![None; program.bindings.len()],
    };

    for binding in &program.bindings {
        let value = lower_expr(&binding.value, &mut lowering);

        lowering.bindings[binding.id.index()] = Some(value);
        lowering.roots.push(value);
    }

    let output =
        lowering.bindings[program.output.index()].expect("typed output must already be lowered");

    Graph {
        nodes: lowering.nodes,
        roots: lowering.roots,
        output,
        render_samples: program.render.render_samples,
    }
}
