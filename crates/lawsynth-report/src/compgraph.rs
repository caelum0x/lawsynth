//! Portable computation-graph export for LawSynth worlds.
//!
//! A [`ComputationGraph`] is an explicit, inspectable directed acyclic graph
//! (DAG) that represents every law of a world as a network of primitive
//! operators. It is the honest, dependency-free stand-in for an ONNX model:
//! emitting a real ONNX protobuf would require an external protobuf writer, so
//! instead we serialize the *same* graph structure ONNX uses — typed input
//! tensors, constant initializers, operator nodes with named inputs, and named
//! outputs — as documented JSON. Each node's [`GraphOp`] maps one-to-one onto a
//! standard ONNX operator, so the artifact is a truthful computation graph a
//! reader can convert to ONNX, not a mislabeled binary.
//!
//! State variables become graph inputs (`var`); parameters become constant
//! initializers (`const`) carrying their inlined value; each law becomes one
//! graph output producing `d(state)/dt`. Shared sub-expressions are interned so
//! the result is a genuine DAG rather than a tree.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_world::World;

use crate::render::python_number;

/// A primitive operator in a [`ComputationGraph`].
///
/// Every variant names the standard ONNX operator it corresponds to, so the
/// graph maps directly onto an ONNX model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphOp {
    /// A graph input tensor (ONNX graph input); carries a state variable name.
    Var,
    /// A constant initializer (ONNX `Constant` / initializer); carries a value.
    Const,
    /// Binary addition (ONNX `Add`).
    Add,
    /// Binary subtraction (ONNX `Sub`).
    Sub,
    /// Binary multiplication (ONNX `Mul`).
    Mul,
    /// Binary division (ONNX `Div`).
    Div,
    /// Exponentiation (ONNX `Pow`).
    Pow,
    /// Unary negation (ONNX `Neg`).
    Neg,
    /// Natural exponential (ONNX `Exp`).
    Exp,
    /// Natural logarithm (ONNX `Log`).
    Log,
    /// Sine (ONNX `Sin`).
    Sin,
    /// Cosine (ONNX `Cos`).
    Cos,
}

impl GraphOp {
    /// The lowercase op-type label used in the JSON `op` field.
    pub fn label(self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Const => "const",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Pow => "pow",
            Self::Neg => "neg",
            Self::Exp => "exp",
            Self::Log => "log",
            Self::Sin => "sin",
            Self::Cos => "cos",
        }
    }

    /// The standard ONNX operator this op maps onto.
    pub fn onnx_op(self) -> &'static str {
        match self {
            Self::Var => "Input",
            Self::Const => "Constant",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::Pow => "Pow",
            Self::Neg => "Neg",
            Self::Exp => "Exp",
            Self::Log => "Log",
            Self::Sin => "Sin",
            Self::Cos => "Cos",
        }
    }
}

/// A single node in a [`ComputationGraph`].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    /// The node's operator.
    pub op: GraphOp,
    /// Indices of the argument nodes (empty for `Var`/`Const`).
    pub inputs: Vec<usize>,
    /// The literal value for a `Const` node.
    pub value: Option<f64>,
    /// The variable/parameter name for a `Var` or named `Const` node.
    pub name: Option<String>,
}

/// An inspectable DAG of a world's derivative laws.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputationGraph {
    /// State-variable graph inputs, in declaration order; each is the node index
    /// of a `Var` node.
    pub inputs: Vec<(String, usize)>,
    /// All nodes, in topological (definition-before-use) order.
    pub nodes: Vec<GraphNode>,
    /// Graph outputs: `(d{state}/dt name, node index)`, in law order.
    pub outputs: Vec<(String, usize)>,
}

/// Builds a computation graph for every continuous law of `world`.
///
/// State variables become `Var` inputs; parameters become named `Const`
/// initializers holding their inlined value; any other referenced symbol
/// (for example an exogenous input) becomes a named `Const` defaulting to 0.
pub fn build_computation_graph(world: &World) -> ComputationGraph {
    let mut builder = GraphBuilder::default();

    // Reserve `Var` nodes for the state inputs first, in declaration order, so
    // the graph inputs are stable and independent of law traversal order.
    let mut inputs = Vec::new();
    for state in world.state_ids() {
        let index = builder.var(state.as_str());
        inputs.push((state.as_str().to_owned(), index));
    }

    // Resolve every referenced symbol: state -> its Var node; parameter -> a
    // Const initializer with the parameter value; any other symbol (for example
    // an exogenous input) -> a named Const defaulting to 0.
    let parameters = world.parameters();
    let is_state: std::collections::BTreeSet<&str> =
        world.state_ids().map(Identifier::as_str).collect();

    let mut outputs = Vec::new();
    for (target, law) in world.laws() {
        let node = builder.insert_expr(&law.expression, &|id| {
            let name = id.as_str();
            if is_state.contains(name) {
                Symbol::State(name.to_owned())
            } else if let Some(parameter) = parameters.get(id) {
                Symbol::Const { name: name.to_owned(), value: parameter.value }
            } else {
                Symbol::Const { name: name.to_owned(), value: 0.0 }
            }
        });
        outputs.push((format!("d{}/dt", target.as_str()), node));
    }

    ComputationGraph { inputs, nodes: builder.nodes, outputs }
}

/// How a referenced symbol resolves inside the graph.
enum Symbol {
    /// A state variable — reuse its `Var` input node.
    State(String),
    /// A constant initializer with a fixed value.
    Const { name: String, value: f64 },
}

#[derive(Default)]
struct GraphBuilder {
    nodes: Vec<GraphNode>,
    /// Interning table keyed by a structural fingerprint, collapsing shared
    /// sub-expressions into a single node (making the result a DAG).
    interned: HashMap<String, usize>,
    /// State-variable name -> its `Var` node index.
    vars: HashMap<String, usize>,
}

impl GraphBuilder {
    fn var(&mut self, name: &str) -> usize {
        if let Some(&index) = self.vars.get(name) {
            return index;
        }
        let index = self.nodes.len();
        self.nodes.push(GraphNode {
            op: GraphOp::Var,
            inputs: Vec::new(),
            value: None,
            name: Some(name.to_owned()),
        });
        self.vars.insert(name.to_owned(), index);
        index
    }

    fn constant(&mut self, value: f64, name: Option<&str>) -> usize {
        let key = match name {
            Some(name) => format!("const:{name}"),
            None => format!("lit:{value:.17e}"),
        };
        if let Some(&index) = self.interned.get(&key) {
            return index;
        }
        let index = self.nodes.len();
        self.nodes.push(GraphNode {
            op: GraphOp::Const,
            inputs: Vec::new(),
            value: Some(value),
            name: name.map(str::to_owned),
        });
        self.interned.insert(key, index);
        index
    }

    fn operator(&mut self, op: GraphOp, inputs: Vec<usize>) -> usize {
        let key = format!("{}:{:?}", op.label(), inputs);
        if let Some(&index) = self.interned.get(&key) {
            return index;
        }
        let index = self.nodes.len();
        self.nodes.push(GraphNode { op, inputs, value: None, name: None });
        self.interned.insert(key, index);
        index
    }

    fn insert_expr(&mut self, expression: &Expr, resolve: &dyn Fn(&Identifier) -> Symbol) -> usize {
        match expression {
            Expr::Constant(value) => self.constant(*value, None),
            Expr::Symbol(id) => match resolve(id) {
                Symbol::State(name) => self.var(&name),
                Symbol::Const { name, value } => self.constant(value, Some(&name)),
            },
            Expr::Unary { operator, operand } => {
                let child = self.insert_expr(operand, resolve);
                let op = match operator {
                    UnaryOperator::Negate => GraphOp::Neg,
                    UnaryOperator::Exp => GraphOp::Exp,
                    UnaryOperator::Log => GraphOp::Log,
                    UnaryOperator::Sin => GraphOp::Sin,
                    UnaryOperator::Cos => GraphOp::Cos,
                };
                self.operator(op, vec![child])
            }
            Expr::Binary { operator, left, right } => {
                let left = self.insert_expr(left, resolve);
                let right = self.insert_expr(right, resolve);
                let op = match operator {
                    BinaryOperator::Add => GraphOp::Add,
                    BinaryOperator::Subtract => GraphOp::Sub,
                    BinaryOperator::Multiply => GraphOp::Mul,
                    BinaryOperator::Divide => GraphOp::Div,
                    BinaryOperator::Power => GraphOp::Pow,
                };
                self.operator(op, vec![left, right])
            }
        }
    }
}

/// Evaluates the graph for a given assignment of state-variable inputs.
///
/// Returns the derivative value of every output, keyed by output name
/// (`d{state}/dt`). This mirrors what an ONNX runtime would compute for the
/// exported graph and is used to validate the export against the engine.
pub fn evaluate_graph(
    graph: &ComputationGraph,
    inputs: &BTreeMap<String, f64>,
) -> BTreeMap<String, f64> {
    let mut values = vec![0.0_f64; graph.nodes.len()];
    for (index, node) in graph.nodes.iter().enumerate() {
        values[index] = match node.op {
            GraphOp::Var => {
                let name = node.name.as_deref().unwrap_or_default();
                *inputs.get(name).unwrap_or(&0.0)
            }
            GraphOp::Const => node.value.unwrap_or(0.0),
            GraphOp::Neg => -values[node.inputs[0]],
            GraphOp::Exp => values[node.inputs[0]].exp(),
            GraphOp::Log => values[node.inputs[0]].ln(),
            GraphOp::Sin => values[node.inputs[0]].sin(),
            GraphOp::Cos => values[node.inputs[0]].cos(),
            GraphOp::Add => values[node.inputs[0]] + values[node.inputs[1]],
            GraphOp::Sub => values[node.inputs[0]] - values[node.inputs[1]],
            GraphOp::Mul => values[node.inputs[0]] * values[node.inputs[1]],
            GraphOp::Div => values[node.inputs[0]] / values[node.inputs[1]],
            GraphOp::Pow => values[node.inputs[0]].powf(values[node.inputs[1]]),
        };
    }
    graph.outputs.iter().map(|(name, index)| (name.clone(), values[*index])).collect()
}

/// Serializes a computation graph as documented LawSynth computation-graph JSON.
///
/// The document is explicitly labeled: it is a computation graph whose nodes map
/// onto ONNX operators, not an ONNX binary. It records the ONNX op mapping inline
/// so a reader can reconstruct an equivalent ONNX model.
pub fn render_computation_graph_json(graph: &ComputationGraph, name: &str) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"format\": \"lawsynth-computation-graph\",");
    let _ = writeln!(out, "  \"format_version\": 1,");
    let _ = writeln!(out, "  \"name\": {},", json_string(name));
    let _ = writeln!(
        out,
        "  \"doc\": \"Inspectable DAG of dx/dt = f(x). NOT an .onnx binary: each node's \\\"onnx_op\\\" names the standard ONNX operator it maps onto, so this graph can be reconstructed as an ONNX model without a protobuf writer. State variables are graph inputs; parameters are constant initializers; each output is one state derivative.\","
    );

    // Graph inputs (state variables).
    out.push_str("  \"inputs\": [\n");
    let inputs: Vec<String> = graph
        .inputs
        .iter()
        .map(|(name, index)| {
            format!(
                "    {{ \"name\": {}, \"node\": {}, \"dtype\": \"float64\", \"shape\": [] }}",
                json_string(name),
                index
            )
        })
        .collect();
    out.push_str(&inputs.join(",\n"));
    out.push_str("\n  ],\n");

    // Nodes.
    out.push_str("  \"nodes\": [\n");
    let nodes: Vec<String> =
        graph.nodes.iter().enumerate().map(|(index, node)| render_node_json(index, node)).collect();
    out.push_str(&nodes.join(",\n"));
    out.push_str("\n  ],\n");

    // Outputs (one per state derivative).
    out.push_str("  \"outputs\": [\n");
    let outputs: Vec<String> = graph
        .outputs
        .iter()
        .map(|(name, index)| {
            format!(
                "    {{ \"name\": {}, \"node\": {}, \"dtype\": \"float64\", \"shape\": [] }}",
                json_string(name),
                index
            )
        })
        .collect();
    out.push_str(&outputs.join(",\n"));
    out.push_str("\n  ]\n");
    out.push_str("}\n");
    out
}

fn render_node_json(index: usize, node: &GraphNode) -> String {
    let inputs: Vec<String> = node.inputs.iter().map(|input| input.to_string()).collect();
    let mut fields = format!(
        "    {{ \"id\": {}, \"op\": {}, \"onnx_op\": {}, \"inputs\": [{}]",
        index,
        json_string(node.op.label()),
        json_string(node.op.onnx_op()),
        inputs.join(", ")
    );
    if let Some(name) = &node.name {
        let _ = write!(fields, ", \"name\": {}", json_string(name));
    }
    if let Some(value) = node.value {
        let _ = write!(fields, ", \"value\": {}", python_number(value));
    }
    fields.push_str(" }");
    fields
}

/// Serializes a string as a JSON string literal with the required escapes.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::{Environment, evaluate};
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn lotka_volterra() -> World {
        // dx/dt = alpha*x - beta*x*y ; dy/dt = delta*x*y - gamma*y
        let sym = |name: &str| Expr::symbol(id(name));
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [
                Parameter::new(id("alpha"), 1.1),
                Parameter::new(id("beta"), 0.4),
                Parameter::new(id("delta"), 0.1),
                Parameter::new(id("gamma"), 0.4),
            ],
            [
                ContinuousLaw::new(
                    id("x"),
                    Expr::difference(
                        Expr::product(sym("alpha"), sym("x")),
                        Expr::product(sym("beta"), Expr::product(sym("x"), sym("y"))),
                    ),
                ),
                ContinuousLaw::new(
                    id("y"),
                    Expr::difference(
                        Expr::product(sym("delta"), Expr::product(sym("x"), sym("y"))),
                        Expr::product(sym("gamma"), sym("y")),
                    ),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn graph_inputs_and_outputs_match_states() {
        let graph = build_computation_graph(&lotka_volterra());
        let input_names: Vec<&str> = graph.inputs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(input_names, ["x", "y"]);
        let output_names: Vec<&str> = graph.outputs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(output_names, ["dx/dt", "dy/dt"]);
    }

    #[test]
    fn shared_subexpressions_are_interned_into_a_dag() {
        // x*y appears in both laws; the Mul node must be shared, not duplicated.
        let graph = build_computation_graph(&lotka_volterra());
        let xy_nodes = graph.nodes.iter().filter(|node| node.op == GraphOp::Mul).count();
        // Muls: (alpha*x), (x*y), (beta*(x*y)), (delta*(x*y)), (gamma*y) = 5,
        // with the single shared (x*y) counted once — proving DAG interning.
        assert_eq!(xy_nodes, 5);
    }

    #[test]
    fn graph_evaluates_to_the_same_derivatives_as_the_expression_engine() {
        let world = lotka_volterra();
        let graph = build_computation_graph(&world);

        for &(x, y) in &[(10.0, 5.0), (0.5, 3.25), (7.0, 0.1)] {
            let inputs = BTreeMap::from([("x".to_owned(), x), ("y".to_owned(), y)]);
            let evaluated = evaluate_graph(&graph, &inputs);

            // Reference: evaluate each law's expression directly with the engine.
            let mut environment = Environment::new();
            environment.insert(id("x"), x);
            environment.insert(id("y"), y);
            for (parameter, value) in
                [("alpha", 1.1), ("beta", 0.4), ("delta", 0.1), ("gamma", 0.4)]
            {
                environment.insert(id(parameter), value);
            }
            for (target, law) in world.laws() {
                let reference = evaluate(&law.expression, &environment).unwrap();
                let output = format!("d{}/dt", target.as_str());
                let graph_value = evaluated[&output];
                assert!(
                    (graph_value - reference).abs() < 1e-12,
                    "output {output}: graph {graph_value} vs engine {reference}"
                );
            }
        }
    }

    #[test]
    fn json_is_labeled_and_maps_to_onnx_ops() {
        let graph = build_computation_graph(&lotka_volterra());
        let json = render_computation_graph_json(&graph, "lotka");
        assert!(json.contains("\"format\": \"lawsynth-computation-graph\""));
        assert!(json.contains("NOT an .onnx binary"));
        assert!(json.contains("\"onnx_op\": \"Mul\""));
        assert!(json.contains("\"onnx_op\": \"Sub\""));
        assert!(json.contains("\"op\": \"var\""));
        assert!(json.contains("\"name\": \"dx/dt\""));
        // A named constant initializer carries both name and value.
        assert!(json.contains("\"name\": \"alpha\""));
        assert!(json.contains("\"value\": 1.1"));
    }
}
