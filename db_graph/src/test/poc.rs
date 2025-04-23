fn ts_poc() -> Result<()> {
    // Example: Parse a Python file
    let language: Language = tree_sitter_rust::LANGUAGE.into();
    let code = r#"fn main() { println!("Hello, world!"); }"#;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(code, None).unwrap();

    println!("Parsed tree: {:?}", tree.root_node());
    println!("Root node: {:?}", tree.root_node().kind());
    println!(
        "Root node start position: {:?}",
        tree.root_node().start_position()
    );
    println!(
        "Root node end position: {:?}",
        tree.root_node().end_position()
    );

    Ok(())
}

fn indra_poc() -> Result<()> {
    // Create an in-memory datastore
    let db: indradb::Database<indradb::MemoryDatastore> = indradb::MemoryDatastore::new_db();

    let rocks_db =
        indradb::RocksdbDatastore::new_db("rocks_db").context("Failed to create RocksDB")?;

    let id1 =
        indradb::Identifier::new("person".to_string()).context("Failed to create identifier")?;

    let id2 =
        indradb::Identifier::new("movie".to_string()).context("Failed to create identifier")?;

    // Create a couple of vertices
    let out_v = indradb::Vertex::new(id1);
    let in_v = indradb::Vertex::new(id2);
    db.create_vertex(&out_v)?;
    db.create_vertex(&in_v)?;

    rocks_db.create_vertex(&out_v)?;
    rocks_db.create_vertex(&in_v)?;

    // Add an edge between the vertices
    let edge = indradb::Edge::new(out_v.id, indradb::Identifier::new("likes")?, in_v.id);
    db.create_edge(&edge)?;
    rocks_db.create_edge(&edge)?;

    // Query for the edge
    let output: Vec<indradb::QueryOutputValue> =
        db.get(indradb::SpecificEdgeQuery::single(edge.clone()))?;

    // Convenience function to extract out the edges from the query results
    let e = indradb::util::extract_edges(output).unwrap();
    println!("Edges: {:?}", e);

    let q1 = indradb::AllVertexQuery;
    let q2 = indradb::AllEdgeQuery;
    println!("Vertices: {:?}", db.get(q1.clone())?);
    println!("Edges: {:?}", db.get(q2.clone())?);

    let dq1 = db.get(q1.clone()).context("Failed to get vertices")?;
    for v in dq1 {
        match v {
            indradb::QueryOutputValue::Vertices(v) => println!("Vertices: {:?}", v),
            _ => println!("Unexpected output"),
        }
    }

    let dq2 = db.get(q2.clone()).context("Failed to get edges")?;
    for e in dq2 {
        match e {
            indradb::QueryOutputValue::Edges(e) => println!("Edges: {:?}", e),
            _ => println!("Unexpected output"),
        }
    }

    Ok(())
}
