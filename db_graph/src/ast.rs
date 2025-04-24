use anyhow::{Context, Result};
use indradb::{Database, Edge, Identifier, MemoryDatastore, Vertex};
use petgraph::{graph::Graph, graph::NodeIndex};
use regex::Regex;
use std::{collections::HashMap, fs};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};
use uuid::Uuid;

pub struct AstProcessor {
    pub db: Database<MemoryDatastore>,
    pub graph: Graph<NodeInfo, &'static str>,
    node_id_map: HashMap<usize, NodeIndex>, // Map from Tree-sitter Node ID to NodeIndex in petgraph
    // Store entity name -> NodeIndex mapping for lookup
    entity_map: HashMap<String, NodeIndex>,
}

// Store both type and name of the node
struct NodeInfo {
    kind: String,
    name: String,
}

impl AstProcessor {
    pub fn new() -> Self {
        AstProcessor {
            db: MemoryDatastore::new_db(),
            graph: Graph::new(),
            node_id_map: HashMap::new(),
            entity_map: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let code = fs::read_to_string(file_path).context("Failed to read file")?;
        let language: Language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&language)?;

        let tree = parser.parse(&code, None).expect("Parsing failed");
        let root_node = tree.root_node();

        self.ast_to_graph(&root_node, &code)?;

        Ok(())
    }

    fn ast_to_graph(
        &mut self,
        root_node: Node,
        code: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query_str = "
            (function_item (identifier) @function)
            (struct_item (type_identifier) @struct)
            (call_expression (identifier) @function_call)
            (struct_expression (path (identifier) @struct_instance))
        ";
        let language: Language = tree_sitter_rust::LANGUAGE.into();
        let query = Query::new(&language, query_str)?;

        // Create a cursor for the root node
        let mut cursor = root_node.walk();
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, root_node, code.as_bytes());

        // First pass: Create all entity nodes
        for match_result in query_cursor.captures(&query, root_node, code.as_bytes()) {
            let (match_obj, capture_index) = match_result;
            let capture = match_obj.captures[capture_index];
            let node = capture.node;

            let node_id = node.id();
            if let Ok(node_text) = node.utf8_text(code.as_bytes()) {
                match capture.index {
                    0 => self.create_vertex(node_id, "Function", node_text),
                    1 => self.create_vertex(node_id, "Struct", node_text),
                    // Handle other captures as needed
                    _ => {}
                }
            }
        }

        // Second pass: Create relationships
        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let node_id = node.id();
                let node_text = node.utf8_text(code.as_bytes())?;

                match capture.index {
                    2 => self.create_edge(node_id, "Calls", node_text),
                    3 => self.create_edge(node_id, "Instantiates", node_text),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn create_vertex(&mut self, node_id: usize, kind: &str, name: &str) -> NodeIndex {
        if !self.node_id_map.contains_key(&node_id) {
            let node_info = NodeInfo {
                kind: kind.to_string(),
                name: name.to_string(),
            };

            let graph_node = self.graph.add_node(node_info);
            self.node_id_map.insert(node_id, graph_node);
            self.entity_map.insert(name.to_string(), graph_node);
        }
        self.node_id_map[&node_id]
    }

    fn create_edge(&mut self, source_node_id: usize, relationship: &str, target_name: &str) {
        // Source is the call site or struct instantiation
        let source_node = self.node_id_map[&source_node_id];

        // Find the target by name (function or struct being referred to)
        if let Some(&target_node) = self.entity_map.get(target_name) {
            self.graph.add_edge(source_node, target_node, relationship);
        }
    }

    fn sanitize_identifier(&self, ident: &str) -> String {
        let re = Regex::new(r"[^A-Za-z0-9_]").unwrap();
        let mut sanitized = re.replace_all(ident, "_").to_string();

        if sanitized.chars().next().map_or(false, |c| c.is_numeric()) {
            sanitized.insert(0, '_');
        }

        sanitized
    }

    pub fn insert_into_indradb(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut indra_vertex_ids = HashMap::<NodeIndex, Uuid>::new();

        // Insert all vertices
        for node_idx in self.graph.node_indices() {
            let node_info = self.graph.node_weight(node_idx).unwrap();
            let sanitized_name = self.sanitize_identifier(&node_info.name);

            let identifier = Identifier::new(&sanitized_name)?;
            let vertex = Vertex::new(identifier);
            let vertex_id = self.db.create_vertex(&vertex)?;

            // Store property for the node type
            // (In a real implementation, you'd use properties)

            indra_vertex_ids.insert(node_idx, vertex_id);
        }

        // Insert all edges
        for edge in self.graph.edge_references() {
            let (source, target, relationship) = (edge.source(), edge.target(), *edge.weight());
            let source_id = indra_vertex_ids[&source];
            let target_id = indra_vertex_ids[&target];

            let edge_label = Identifier::new(relationship)?;
            let edge = Edge::new(source_id, edge_label, target_id);
            self.db.create_edge(&edge)?;
        }

        Ok(())
    }
}
