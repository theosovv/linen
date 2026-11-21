use linen::ir::{Graph, Node, NodeId, NodeOp, NodeOrigin, ValueType, lower, verify};
use linen::sema::{BuiltinId, ResourceModel, RuntimeCapability, builtin_descriptor};
use linen::{CompiledProgram, RenderConfig, SourceMap, SourceSpan, compile};

fn context() -> (SourceMap, SourceSpan) {
    let mut sources = SourceMap::new();
    let source = sources.add_source("ir.ln", "0");
    let span = SourceSpan::new(source, 0, 1).expect("the test span is valid");
    (sources, span)
}

fn node(index: usize, op: NodeOp, result_type: ValueType, span: SourceSpan) -> Node {
    Node {
        id: NodeId::new(index),
        op,
        result_type,
        origin: NodeOrigin { primary: span },
    }
}

fn valid_graph(span: SourceSpan) -> Graph {
    Graph {
        nodes: vec![
            node(0, NodeOp::ConstFloat(480.0), ValueType::Float, span),
            node(
                1,
                NodeOp::LiftFloatToSig(NodeId::new(0)),
                ValueType::Sig,
                span,
            ),
            node(2, NodeOp::Sine(NodeId::new(1)), ValueType::Sig, span),
        ],
        roots: vec![NodeId::new(2)],
        output: NodeId::new(2),
        render_samples: 48,
    }
}

fn compile_program(source: &str) -> CompiledProgram {
    compile(source, RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the source should compile")
}

#[test]
fn spec_5_6_lowering_builds_explicit_nodes_roots_and_sharing() {
    let program = compile_program(
        "let base = 440;\nlet wave = sine(base);\nlet out = wave * 0.1;\nrender 1ms;\n",
    );
    let operations: Vec<_> = program
        .ir
        .nodes
        .iter()
        .map(|node| node.op.clone())
        .collect();

    assert_eq!(
        operations,
        vec![
            NodeOp::ConstFloat(440.0),
            NodeOp::LiftFloatToSig(NodeId::new(0)),
            NodeOp::Sine(NodeId::new(1)),
            NodeOp::ConstFloat(0.1),
            NodeOp::LiftFloatToSig(NodeId::new(3)),
            NodeOp::Mul(NodeId::new(2), NodeId::new(4)),
        ]
    );
    assert_eq!(
        program.ir.roots,
        [NodeId::new(0), NodeId::new(2), NodeId::new(5)]
    );
    assert_eq!(program.ir.output, NodeId::new(5));
    assert_eq!(program.ir.render_samples, 48);
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .map(|node| node.result_type)
            .collect::<Vec<_>>(),
        [
            ValueType::Float,
            ValueType::Sig,
            ValueType::Sig,
            ValueType::Float,
            ValueType::Sig,
            ValueType::Sig,
        ]
    );
}

#[test]
fn adr_0004_aliases_and_groups_share_nodes_without_extra_state() {
    let program =
        compile_program("let value = 1;\nlet alias = value;\nlet out = (alias);\nrender 1ms;\n");

    assert_eq!(program.ir.nodes.len(), 2);
    assert_eq!(program.ir.nodes[0].op, NodeOp::ConstFloat(1.0));
    assert_eq!(
        program.ir.nodes[1].op,
        NodeOp::LiftFloatToSig(NodeId::new(0))
    );
    assert_eq!(
        program.ir.roots,
        [NodeId::new(0), NodeId::new(0), NodeId::new(1)]
    );
}

#[test]
fn spec_5_6_every_lowered_node_has_a_valid_source_origin() {
    let outcome = compile(
        include_str!("../fixtures/valid/stage-03-shared-sine.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let program = outcome.result.as_ref().expect("the fixture should compile");

    for node in &program.ir.nodes {
        let span = node.origin.primary;
        let source = sources_get(&outcome.sources, span);
        assert!(source.text().is_char_boundary(span.start()));
        assert!(source.text().is_char_boundary(span.end()));
        assert!(span.end() <= source.text().len());
    }
    verify(&program.ir, &outcome.sources).expect("lowered origins must verify");
}

fn sources_get(sources: &SourceMap, span: SourceSpan) -> &linen::SourceFile {
    sources
        .get(span.source())
        .expect("the origin source must exist")
}

#[test]
fn adr_0004_direct_lower_entry_point_uses_the_same_graph_contract() {
    let program = compile_program("let out = sine(480); render 1ms;");
    let graph = lower(&program.typed);
    assert_eq!(graph, program.ir);
}

#[test]
fn spec_5_6_verifier_accepts_a_canonical_typed_graph() {
    let (sources, span) = context();
    verify(&valid_graph(span), &sources).expect("the manually built graph is valid");
}

#[test]
fn spec_5_6_verifier_rejects_node_id_mismatch() {
    let (sources, span) = context();
    let mut graph = valid_graph(span);
    graph.nodes[1].id = NodeId::new(7);

    assert_eq!(
        verify(&graph, &sources),
        Err(linen::ir::IrVerifyError::NodeIdMismatch {
            index: 1,
            actual: NodeId::new(7),
        })
    );
}

#[test]
fn spec_5_6_verifier_rejects_unknown_and_non_topological_inputs() {
    let (sources, span) = context();
    let mut unknown = valid_graph(span);
    unknown.nodes[1].op = NodeOp::LiftFloatToSig(NodeId::new(99));
    assert_eq!(
        verify(&unknown, &sources),
        Err(linen::ir::IrVerifyError::UnknownInput {
            node: NodeId::new(1),
            input: NodeId::new(99),
        })
    );

    let cyclic = Graph {
        nodes: vec![node(0, NodeOp::Neg(NodeId::new(0)), ValueType::Sig, span)],
        roots: vec![NodeId::new(0)],
        output: NodeId::new(0),
        render_samples: 1,
    };
    assert_eq!(
        verify(&cyclic, &sources),
        Err(linen::ir::IrVerifyError::NonTopologicalInput {
            node: NodeId::new(0),
            input: NodeId::new(0),
        })
    );
}

#[test]
fn spec_5_6_verifier_rejects_input_and_result_type_mismatches() {
    let (sources, span) = context();
    let mut input = valid_graph(span);
    input.nodes[2].op = NodeOp::Sine(NodeId::new(0));
    assert_eq!(
        verify(&input, &sources),
        Err(linen::ir::IrVerifyError::InputTypeMismatch {
            node: NodeId::new(2),
            input: NodeId::new(0),
            expected: ValueType::Sig,
            actual: ValueType::Float,
        })
    );

    let mut result = valid_graph(span);
    result.nodes[1].result_type = ValueType::Float;
    assert_eq!(
        verify(&result, &sources),
        Err(linen::ir::IrVerifyError::ResultTypeMismatch {
            node: NodeId::new(1),
            expected: ValueType::Sig,
            actual: ValueType::Float,
        })
    );
}

#[test]
fn spec_5_6_verifier_rejects_nonfinite_constants() {
    let (sources, span) = context();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut graph = valid_graph(span);
        graph.nodes[0].op = NodeOp::ConstFloat(value);
        assert_eq!(
            verify(&graph, &sources),
            Err(linen::ir::IrVerifyError::NonFiniteConstant {
                node: NodeId::new(0),
            })
        );
    }
}

#[test]
fn spec_5_6_verifier_rejects_invalid_roots_and_output() {
    let (sources, span) = context();
    let mut empty = valid_graph(span);
    empty.roots.clear();
    assert_eq!(
        verify(&empty, &sources),
        Err(linen::ir::IrVerifyError::EmptyRoots)
    );

    let mut unknown_root = valid_graph(span);
    unknown_root.roots.push(NodeId::new(99));
    assert_eq!(
        verify(&unknown_root, &sources),
        Err(linen::ir::IrVerifyError::UnknownRoot {
            root_index: 1,
            root: NodeId::new(99),
        })
    );

    let mut unknown_output = valid_graph(span);
    unknown_output.output = NodeId::new(99);
    assert_eq!(
        verify(&unknown_output, &sources),
        Err(linen::ir::IrVerifyError::UnknownOutput(NodeId::new(99)))
    );

    let mut not_root = valid_graph(span);
    not_root.roots = vec![NodeId::new(1)];
    assert_eq!(
        verify(&not_root, &sources),
        Err(linen::ir::IrVerifyError::OutputNotRoot(NodeId::new(2)))
    );

    let mut wrong_type = valid_graph(span);
    wrong_type.output = NodeId::new(0);
    wrong_type.roots = vec![NodeId::new(0), NodeId::new(2)];
    assert_eq!(
        verify(&wrong_type, &sources),
        Err(linen::ir::IrVerifyError::OutputTypeMismatch(
            ValueType::Float
        ))
    );
}

#[test]
fn spec_5_6_verifier_rejects_unreachable_nodes() {
    let (sources, span) = context();
    let mut graph = valid_graph(span);
    graph
        .nodes
        .push(node(3, NodeOp::ConstFloat(1.0), ValueType::Float, span));

    assert_eq!(
        verify(&graph, &sources),
        Err(linen::ir::IrVerifyError::UnreachableNode(NodeId::new(3)))
    );
}

#[test]
fn spec_5_6_verifier_rejects_unknown_out_of_bounds_and_non_utf8_origins() {
    let (sources, span) = context();
    let mut other = SourceMap::new();
    other.add_source("zero.ln", "0");
    let unknown = other.add_source("one.ln", "1");
    let mut graph = valid_graph(span);
    graph.nodes[0].origin.primary = SourceSpan::empty(unknown, 0);
    assert_eq!(
        verify(&graph, &sources),
        Err(linen::ir::IrVerifyError::InvalidOrigin {
            node: NodeId::new(0),
            span: SourceSpan::empty(unknown, 0),
        })
    );

    let mut out_of_bounds = valid_graph(span);
    out_of_bounds.nodes[0].origin.primary =
        SourceSpan::new(span.source(), 0, 2).expect("reversed offsets are not used");
    assert_eq!(
        verify(&out_of_bounds, &sources),
        Err(linen::ir::IrVerifyError::InvalidOrigin {
            node: NodeId::new(0),
            span: SourceSpan::new(span.source(), 0, 2).unwrap(),
        })
    );

    let mut unicode = SourceMap::new();
    let unicode_id = unicode.add_source("unicode.ln", "ё");
    let non_boundary = SourceSpan::new(unicode_id, 0, 1).expect("offsets are ordered");
    let graph = valid_graph(non_boundary);
    assert_eq!(
        verify(&graph, &unicode),
        Err(linen::ir::IrVerifyError::InvalidOrigin {
            node: NodeId::new(0),
            span: non_boundary,
        })
    );
}

#[test]
fn spec_9_every_ir_op_has_explicit_capability_and_resource_metadata() {
    let scalar = ResourceModel {
        scalar_registers: 1,
        audio_registers: 0,
        state_slots: 0,
        state_bytes: 0,
    };
    let audio = ResourceModel {
        scalar_registers: 0,
        audio_registers: 1,
        state_slots: 0,
        state_bytes: 0,
    };
    let cases = [
        (NodeOp::ConstFloat(1.0), ValueType::Float, scalar),
        (
            NodeOp::LiftFloatToSig(NodeId::new(0)),
            ValueType::Sig,
            audio,
        ),
        (NodeOp::Neg(NodeId::new(0)), ValueType::Float, scalar),
        (NodeOp::Neg(NodeId::new(0)), ValueType::Sig, audio),
        (
            NodeOp::Add(NodeId::new(0), NodeId::new(0)),
            ValueType::Float,
            scalar,
        ),
        (
            NodeOp::Sub(NodeId::new(0), NodeId::new(0)),
            ValueType::Sig,
            audio,
        ),
        (
            NodeOp::Mul(NodeId::new(0), NodeId::new(0)),
            ValueType::Sig,
            audio,
        ),
        (
            NodeOp::Div(NodeId::new(0), NodeId::new(0)),
            ValueType::Float,
            scalar,
        ),
    ];

    for (op, result_type, resources) in cases {
        let descriptor = op.descriptor(result_type);
        assert_eq!(descriptor.capability, RuntimeCapability::CoreAudio);
        assert_eq!(descriptor.resources, resources);
    }

    let sine = NodeOp::Sine(NodeId::new(0)).descriptor(ValueType::Sig);
    let builtin = builtin_descriptor(BuiltinId::SINE).expect("sine descriptor exists");
    assert_eq!(sine.capability, RuntimeCapability::CoreAudio);
    assert_eq!(sine.capability, builtin.capability);
    assert_eq!(sine.resources, builtin.resources);
    assert_eq!(sine.resources.state_slots, 1);
    assert_eq!(sine.resources.state_bytes, size_of::<f64>());
}
