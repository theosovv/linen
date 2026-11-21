use std::{error::Error, fmt};

use crate::{
    SourceMap, SourceSpan,
    ir::{Node, NodeOp},
};

use super::{Graph, NodeId, ValueType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrVerifyError {
    NodeIdMismatch {
        index: usize,
        actual: NodeId,
    },
    UnknownInput {
        node: NodeId,
        input: NodeId,
    },
    NonTopologicalInput {
        node: NodeId,
        input: NodeId,
    },
    InputTypeMismatch {
        node: NodeId,
        input: NodeId,
        expected: ValueType,
        actual: ValueType,
    },
    ResultTypeMismatch {
        node: NodeId,
        expected: ValueType,
        actual: ValueType,
    },
    NonFiniteConstant {
        node: NodeId,
    },
    EmptyRoots,
    UnknownRoot {
        root_index: usize,
        root: NodeId,
    },
    UnknownOutput(NodeId),
    OutputNotRoot(NodeId),
    OutputTypeMismatch(ValueType),
    UnreachableNode(NodeId),
    InvalidOrigin {
        node: NodeId,
        span: SourceSpan,
    },
    InvalidInlineOrigin {
        node: NodeId,
        inline_index: usize,
        span: SourceSpan,
    },
}

impl fmt::Display for IrVerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid signal IR: {self:?}")
    }
}

impl Error for IrVerifyError {}

fn input_type(graph: &Graph, node: &Node, input: NodeId) -> Result<ValueType, IrVerifyError> {
    let input_node = graph
        .nodes
        .get(input.index())
        .ok_or(IrVerifyError::UnknownInput {
            node: node.id,
            input,
        })?;

    if input.index() >= node.id.index() {
        return Err(IrVerifyError::NonTopologicalInput {
            node: node.id,
            input,
        });
    }

    Ok(input_node.result_type)
}

fn origin_is_valid(sources: &SourceMap, span: SourceSpan) -> bool {
    let Some(source) = sources.get(span.source()) else {
        return false;
    };
    let text = source.text();

    span.start() <= span.end()
        && span.end() <= text.len()
        && text.is_char_boundary(span.start())
        && text.is_char_boundary(span.end())
}

fn for_each_input(op: &NodeOp, mut visit: impl FnMut(NodeId)) {
    match op {
        NodeOp::ConstFloat(_) => {}
        NodeOp::LiftFloatToSig(input) | NodeOp::Sine(input) | NodeOp::Neg(input) => visit(*input),
        NodeOp::Add(left, right)
        | NodeOp::Sub(left, right)
        | NodeOp::Mul(left, right)
        | NodeOp::Div(left, right) => {
            visit(*left);
            visit(*right);
        }
    }
}

pub fn verify(graph: &Graph, sources: &SourceMap) -> Result<(), IrVerifyError> {
    for (index, node) in graph.nodes.iter().enumerate() {
        let expected = NodeId::new(index);

        if node.id != expected {
            return Err(IrVerifyError::NodeIdMismatch {
                index,
                actual: node.id,
            });
        }

        match node.op {
            NodeOp::ConstFloat(value) => {
                if !value.is_finite() {
                    return Err(IrVerifyError::NonFiniteConstant { node: node.id });
                }

                if node.result_type != ValueType::Float {
                    return Err(IrVerifyError::ResultTypeMismatch {
                        node: node.id,
                        expected: ValueType::Float,
                        actual: node.result_type,
                    });
                }
            }
            NodeOp::LiftFloatToSig(input_id) => {
                let input = input_type(graph, node, input_id)?;

                if input != ValueType::Float {
                    return Err(IrVerifyError::InputTypeMismatch {
                        node: node.id,
                        input: input_id,
                        expected: ValueType::Float,
                        actual: input,
                    });
                }

                if node.result_type != ValueType::Sig {
                    return Err(IrVerifyError::ResultTypeMismatch {
                        node: node.id,
                        expected: ValueType::Sig,
                        actual: node.result_type,
                    });
                }
            }
            NodeOp::Neg(input_id) => {
                let input = input_type(graph, node, input_id)?;

                if node.result_type != input {
                    return Err(IrVerifyError::ResultTypeMismatch {
                        node: node.id,
                        expected: input,
                        actual: node.result_type,
                    });
                }
            }
            NodeOp::Add(left, right)
            | NodeOp::Sub(left, right)
            | NodeOp::Mul(left, right)
            | NodeOp::Div(left, right) => {
                let left_input = input_type(graph, node, left)?;
                let right_input = input_type(graph, node, right)?;

                if left_input != right_input {
                    return Err(IrVerifyError::InputTypeMismatch {
                        node: node.id,
                        input: right,
                        expected: left_input,
                        actual: right_input,
                    });
                }

                if node.result_type != left_input || node.result_type != right_input {
                    return Err(IrVerifyError::ResultTypeMismatch {
                        node: node.id,
                        expected: left_input,
                        actual: node.result_type,
                    });
                }
            }
            NodeOp::Sine(frequency_id) => {
                let input = input_type(graph, node, frequency_id)?;

                if input != ValueType::Sig {
                    return Err(IrVerifyError::InputTypeMismatch {
                        node: node.id,
                        input: frequency_id,
                        expected: ValueType::Sig,
                        actual: input,
                    });
                }

                if node.result_type != ValueType::Sig {
                    return Err(IrVerifyError::ResultTypeMismatch {
                        node: node.id,
                        expected: ValueType::Sig,
                        actual: node.result_type,
                    });
                }
            }
        }

        if !origin_is_valid(sources, node.origin.primary) {
            return Err(IrVerifyError::InvalidOrigin {
                node: node.id,
                span: node.origin.primary,
            });
        }

        for (inline_index, call) in node.origin.inlined_from.iter().enumerate() {
            if !origin_is_valid(sources, call.call_span) {
                return Err(IrVerifyError::InvalidInlineOrigin {
                    node: node.id,
                    inline_index,
                    span: call.call_span,
                });
            }
        }
    }

    if graph.roots.is_empty() {
        return Err(IrVerifyError::EmptyRoots);
    }

    for (root_index, &root) in graph.roots.iter().enumerate() {
        if root.index() >= graph.nodes.len() {
            return Err(IrVerifyError::UnknownRoot { root_index, root });
        }
    }

    let output_node = graph
        .nodes
        .get(graph.output.index())
        .ok_or(IrVerifyError::UnknownOutput(graph.output))?;

    if !graph.roots.contains(&graph.output) {
        return Err(IrVerifyError::OutputNotRoot(graph.output));
    }

    if output_node.result_type != ValueType::Sig {
        return Err(IrVerifyError::OutputTypeMismatch(output_node.result_type));
    }

    let mut reachable = vec![false; graph.nodes.len()];

    for &root in &graph.roots {
        reachable[root.index()] = true;
    }

    for node in graph.nodes.iter().rev() {
        if !reachable[node.id.index()] {
            continue;
        }

        for_each_input(&node.op, |input| {
            reachable[input.index()] = true;
        });
    }

    for node in &graph.nodes {
        if !reachable[node.id.index()] {
            return Err(IrVerifyError::UnreachableNode(node.id));
        }
    }

    Ok(())
}
