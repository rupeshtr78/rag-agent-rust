use anyhow::{Context, Ok, Result};
use indradb::{Database, Edge, Identifier, MemoryDatastore, Vertex};
use petgraph::visit::EdgeRef;
use petgraph::{graph::EdgeReference, graph::Graph, graph::NodeIndex, Direction};
use regex::Regex;
use std::collections::HashMap;
use std::{fs, path::Path};
use tree_sitter::Language;
use tree_sitter::{Node, Parser, Tree};

// mod ast;

fn main() -> Result<()> {
    // indra_poc().context("Error runnning indra poc")?;

    // ts_poc().context("Error running tree-sitter poc")?;
    let db: Database<MemoryDatastore> = MemoryDatastore::new_db();
    let file_path = "app/src/main.rs";

    parse_file_to_indradb(file_path, &db).context("Error loading file to IndraDB")?;
    Ok(())
}

// Read, parse, and populate IndraDB from given source file
fn parse_file_to_indradb(
    file_path: impl AsRef<Path>,
    db: &Database<MemoryDatastore>,
) -> Result<()> {
    // Read full file content as a String
    let code = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read file {}", file_path.as_ref().display()))?;

    // Setup tree-sitter parser
    let language: Language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .context("Failed to set language")?;

    // Parse the code into an AST
    let tree = parser.parse(&code, None).context("Failed to parse code")?;

    let root_node = tree.root_node();
    println!("Root node kind: {}", root_node.kind());

    // // Start recursively inserting nodes into IndraDB graph
    // insert_node_recursive(db, &root_node, None).context("Failed to insert nodes into IndraDB")?;

    // Generate the petgraph from the AST
    let pet_graph = ast_to_graph(&root_node);

    // Insert the petgraph into IndraDB
    insert_petgraph_to_indradb(db, &pet_graph).context("Failed to insert petgraph into IndraDB")?;

    // Query
    let q1 = indradb::AllVertexQuery;
    let q2 = indradb::AllEdgeQuery;
    println!("Vertices: {:?}", db.get(q1.clone())?);
    println!("Edges: {:?}", db.get(q2.clone())?);

    let dq1 = db.get(q1.clone()).context("Failed to get vertices")?;
    // for v in dq1 {
    //     match v {
    //         indradb::QueryOutputValue::Vertices(v) => {
    //             // println!("Vertices: {:?}", v);
    //             for k in v {
    //                 // let identifier = k.t;
    //                 println!("Vertex Identifier: {:?}", k);
    //             }
    //         }
    //         _ => println!("Unexpected output"),
    //     }
    // }

    let dq2 = db.get(q2.clone()).context("Failed to get edges")?;
    for e in dq2 {
        match e {
            indradb::QueryOutputValue::Edges(e) => println!("Edges: {:?}", e),
            _ => println!("Unexpected output"),
        }
    }

    Ok(())
}

// Recursive function to traverse AST and insert nodes into IndraDB as vertices and edges
fn insert_node_recursive(
    db: &Database<MemoryDatastore>,
    node: &Node,
    parent_vertex_id: Option<uuid::Uuid>,
) -> Result<()> {
    // Each node kind becomes a vertex
    // println!("Node kind: {}", node.kind());
    // Sanitize the identifier string
    let sanitized_kind = sanitize_identifier(node.kind());
    println!("Sanitized node kind: {} to {}", node.kind(), sanitized_kind);

    let identifier = Identifier::new(sanitized_kind).context(format!(
        "Failed to create identifier for node kind: {}",
        node.kind()
    ))?;
    let vertex = Vertex::new(identifier);
    db.create_vertex(&vertex)
        .with_context(|| format!("Failed to create vertex for node kind: {}", node.kind()))?;

    // If there is a parent node, create an edge from parent to current node
    if let Some(parent_id) = parent_vertex_id {
        let edge_label = Identifier::new("child").context("Failed to create edge label")?;
        let edge = Edge::new(parent_id, edge_label, vertex.id);
        db.create_edge(&edge).context("Failed to create edge")?;
    }

    // Recursively insert child nodes
    for child in node.children(&mut node.walk()) {
        insert_node_recursive(db, &child, Some(vertex.id))?;
    }

    Ok(())
}

// Sanitize node kind to be a valid IndraDB identifier
fn sanitize_identifier(ident: &str) -> String {
    // Replace all non-identifier characters with underscores '_'
    // Identifier must start with letter or underscore (_, a-z, A-Z), and subsequent chars can include digits(0-9)
    let re = Regex::new(r"[^A-Za-z0-9_]").unwrap();
    let mut sanitized = re.replace_all(ident, "_").to_string();

    // If the string starts with digits, prepend an underscore
    if sanitized.chars().next().map_or(false, |c| c.is_numeric()) {
        sanitized.insert(0, '_');
    }

    sanitized
}

// Function to convert AST to petgraph
fn ast_to_graph(root_node: &Node) -> Graph<String, &'static str> {
    let mut graph = Graph::new();
    let mut stack = Vec::<(Node, Option<NodeIndex>)>::new(); // (current_node, parent_graph_node)

    // Push the root node and None (no parent) onto the stack
    stack.push((root_node.clone(), None));

    while let Some((node, parent)) = stack.pop() {
        // Create a graph node for this AST element
        let node_label = node.kind().to_string();
        let graph_node = graph.add_node(node_label);

        // Connect to parent (if exists)
        if let Some(parent_id) = parent {
            graph.add_edge(parent_id, graph_node, "child");
        }

        // Push children to stack, cloning the Node
        for child in node.children(&mut node.walk()) {
            stack.push((child.clone(), Some(graph_node)));
        }
    }

    graph
}

// Function to insert petgraph into IndraDB
fn insert_petgraph_to_indradb(
    db: &Database<MemoryDatastore>,
    graph: &Graph<String, &'static str>,
) -> Result<()> {
    // Map petgraph NodeIndex to UUID for IndraDB
    let mut node_id_map = HashMap::<NodeIndex, uuid::Uuid>::new();

    // Insert all vertices
    for node in graph.node_indices() {
        let node_label = graph.node_weight(node).unwrap().clone();
        let sanitized_kind = sanitize_identifier(&node_label);
        println!("Sanitized node kind: {} to {}", node_label, sanitized_kind);

        let identifier = Identifier::new(sanitized_kind).context(format!(
            "Failed to create identifier for node kind: {}",
            node_label
        ))?;
        let vertex = Vertex::new(identifier);
        db.create_vertex(&vertex)
            .with_context(|| format!("Failed to create vertex for node kind: {}", node_label))?;

        node_id_map.insert(node, vertex.id);
    }

    // Insert all edges
    for edge in graph.edge_references() {
        let (source, target, label) = (edge.source(), edge.target(), edge.weight());
        let source_id = node_id_map[&source];
        let target_id = node_id_map[&target];

        let edge_label = Identifier::new(*label).context("Failed to create edge label")?;
        let edge = Edge::new(source_id, edge_label, target_id);
        db.create_edge(&edge).context("Failed to create edge")?;
    }

    Ok(())
}
