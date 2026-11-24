use std::{error::Error, fmt};

use crate::{
    SourceMap, SourceSpan,
    ir::{Node, NodeOp},
};

use super::{Graph, NodeId, ValueType, graph::const_float_value};

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
    InvalidNoiseSeed {
        node: NodeId,
    },
    UnsupportedStereoOperation {
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

fn expect_input(
    graph: &Graph,
    node: &Node,
    input: NodeId,
    expected: ValueType,
) -> Result<(), IrVerifyError> {
    let actual = input_type(graph, node, input)?;
    if actual != expected {
        return Err(IrVerifyError::InputTypeMismatch {
            node: node.id,
            input,
            expected,
            actual,
        });
    }
    Ok(())
}

fn expect_result(node: &Node, expected: ValueType) -> Result<(), IrVerifyError> {
    if node.result_type != expected {
        return Err(IrVerifyError::ResultTypeMismatch {
            node: node.id,
            expected,
            actual: node.result_type,
        });
    }
    Ok(())
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
        NodeOp::LiftFloatToSig(input)
        | NodeOp::Sine(input)
        | NodeOp::Phasor(input)
        | NodeOp::Saw(input)
        | NodeOp::Square(input)
        | NodeOp::SawPolyblep(input)
        | NodeOp::SquarePolyblep(input)
        | NodeOp::Triangle(input)
        | NodeOp::Neg(input) => visit(*input),
        NodeOp::Add(left, right)
        | NodeOp::Sub(left, right)
        | NodeOp::Mul(left, right)
        | NodeOp::Div(left, right)
        | NodeOp::Stereo(left, right)
        | NodeOp::Pan(left, right) => {
            visit(*left);
            visit(*right);
        }
        NodeOp::Noise { seed, .. } => visit(*seed),
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

        match &node.op {
            NodeOp::ConstFloat(value) => {
                if !value.is_finite() {
                    return Err(IrVerifyError::NonFiniteConstant { node: node.id });
                }
                expect_result(node, ValueType::Float)?;
            }
            NodeOp::LiftFloatToSig(input) => {
                expect_input(graph, node, *input, ValueType::Float)?;
                expect_result(node, ValueType::Sig)?;
            }
            NodeOp::Neg(input) => {
                let input_type = input_type(graph, node, *input)?;
                if input_type == ValueType::Stereo {
                    return Err(IrVerifyError::UnsupportedStereoOperation { node: node.id });
                }
                expect_result(node, input_type)?;
            }
            NodeOp::Add(left, right)
            | NodeOp::Sub(left, right)
            | NodeOp::Mul(left, right)
            | NodeOp::Div(left, right) => {
                let left_input = input_type(graph, node, *left)?;
                let right_input = input_type(graph, node, *right)?;

                if left_input == ValueType::Stereo || right_input == ValueType::Stereo {
                    return Err(IrVerifyError::UnsupportedStereoOperation { node: node.id });
                }
                if left_input != right_input {
                    return Err(IrVerifyError::InputTypeMismatch {
                        node: node.id,
                        input: *right,
                        expected: left_input,
                        actual: right_input,
                    });
                }
                expect_result(node, left_input)?;
            }
            NodeOp::Sine(frequency)
            | NodeOp::Phasor(frequency)
            | NodeOp::Saw(frequency)
            | NodeOp::Square(frequency)
            | NodeOp::SawPolyblep(frequency)
            | NodeOp::SquarePolyblep(frequency)
            | NodeOp::Triangle(frequency) => {
                expect_input(graph, node, *frequency, ValueType::Sig)?;
                expect_result(node, ValueType::Sig)?;
            }
            NodeOp::Stereo(left, right) => {
                expect_input(graph, node, *left, ValueType::Sig)?;
                expect_input(graph, node, *right, ValueType::Sig)?;
                expect_result(node, ValueType::Stereo)?;
            }
            NodeOp::Pan(input, position) => {
                expect_input(graph, node, *input, ValueType::Sig)?;
                expect_input(graph, node, *position, ValueType::Sig)?;
                expect_result(node, ValueType::Stereo)?;
            }
            NodeOp::Noise { seed, stream } => {
                expect_input(graph, node, *seed, ValueType::Float)?;
                expect_result(node, ValueType::Sig)?;
                let Some(value) = const_float_value(&graph.nodes, *seed) else {
                    return Err(IrVerifyError::InvalidNoiseSeed { node: node.id });
                };
                if !valid_noise_seed(value) || value as u64 != stream.seed() {
                    return Err(IrVerifyError::InvalidNoiseSeed { node: node.id });
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

    if output_node.result_type == ValueType::Float {
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

fn valid_noise_seed(value: f64) -> bool {
    value.is_finite() && (0.0..=9_007_199_254_740_991.0).contains(&value) && value.fract() == 0.0
}
