//! Lowering from Core AST to Signal Graph.

use crate::core_lang::{Atom, CoreExpr};
use crate::signal_graph::{Node, SignalGraph};

/// Lower a Core expression into a SignalGraph.
pub fn lower_expr(expr: &CoreExpr) -> SignalGraph {
    let mut graph = SignalGraph::new();
    let root = lower_expr_into(&mut graph, expr);
    graph.set_outputs(vec![root]);
    graph
}

fn lower_expr_into(graph: &mut SignalGraph, expr: &CoreExpr) -> usize {
    match expr {
        CoreExpr::Atom(Atom::Literal(crate::ast::Literal::Float(f)), _) => {
            graph.add_node(Node::Constant(*f))
        }
        CoreExpr::Apply(func, args, _) => {
            let func_name = match func {
                Atom::Var(name) => name.as_str(),
                _ => return graph.add_node(Node::Constant(0.0)), // placeholder
            };

            let arg_ids: Vec<_> = args.iter().map(|a| lower_atom(graph, a)).collect();

            match func_name {
                "sin" => graph.add_node(Node::Oscillator(
                    crate::signal_graph::OscType::Sine,
                    arg_ids,
                )),
                "saw" => {
                    graph.add_node(Node::Oscillator(crate::signal_graph::OscType::Saw, arg_ids))
                }
                "square" => graph.add_node(Node::Oscillator(
                    crate::signal_graph::OscType::Square,
                    arg_ids,
                )),
                "triangle" => graph.add_node(Node::Oscillator(
                    crate::signal_graph::OscType::Triangle,
                    arg_ids,
                )),
                "gain" => graph.add_node(Node::Gain(arg_ids)),
                "mix" => graph.add_node(Node::Mix(arg_ids)),
                "lowpass" => graph.add_node(Node::Filter(
                    crate::signal_graph::FilterType::Lowpass,
                    arg_ids,
                )),
                "highpass" => graph.add_node(Node::Filter(
                    crate::signal_graph::FilterType::Highpass,
                    arg_ids,
                )),
                _ => graph.add_node(Node::Constant(0.0)), // placeholder for unknown functions
            }
        }
        _ => graph.add_node(Node::Constant(0.0)), // placeholder
    }
}

fn lower_atom(graph: &mut SignalGraph, atom: &Atom) -> usize {
    match atom {
        Atom::Literal(crate::ast::Literal::Float(f)) => graph.add_node(Node::Constant(*f)),
        _ => graph.add_node(Node::Constant(0.0)), // placeholder
    }
}
