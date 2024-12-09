use log::{debug, error, info};
use pgvector::Vector;
use postgres::{config, Client, Config, NoTls};
use std::{error::Error, time::Duration};

pub fn pg_client() -> Result<Client, Box<dyn Error>> {
    let mut config = Config::new();
    config
        .host("10.0.0.213")
        .port(5555)
        .user("rupesh")
        .dbname("vectordb")
        .connect_timeout(Duration::from_secs(5));

    let client = config.connect(NoTls)?;

    Ok(client)
}

pub fn select_embeddings(client: &mut Client) -> Result<(), Box<dyn Error>> {
    info!("Select method started");

    let query = "SELECT id, text FROM embeddings";
    let rows = client.query(query, &[]);
    match rows {
        Ok(rows) => {
            for row in rows {
                let id: i32 = row.get(0);
                let text: &str = row.get(1);
                info!("id: {}, name: {}", id, text);
            }

            info!("Select statement successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    };

    info!("Select method successful");

    Ok(())
}

pub fn create_table(
    pg_client: &mut Client,
    table: &str,
    dimension: i32,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;

    let drop_query = format!("DROP TABLE IF EXISTS {}", table);
    transaction.execute(&drop_query, &[])?;
    info!("Table dropped: {}", table);

    let query = format!(
        "CREATE TABLE {} (id bigserial PRIMARY KEY, content text, embedding vector({}))",
        table, dimension
    );
    transaction.execute(&query, &[])?;

    info!("Table created: {}", table);
    transaction.commit()?;

    return Ok(());
}

// input []string, embeddings [][]float32, conn *pgx.Conn
pub fn load_vector_data(
    pg_client: &mut Client,
    table: &str,
    input: Vec<String>,
    embeddings: Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;
    let query = format!("INSERT INTO {} (content, embedding) VALUES ($1, $2)", table);

    // convert input to pg vector
    let pgv = embeddings
        .iter()
        .map(|v| Vector::from(v.clone()))
        .collect::<Vec<Vector>>();

    // iterate over input and embeddings
    for i in 0..input.len() {
        let content = &input[i];
        let embedding = &pgv[i];
        debug!("Content: {}, Embedding: {:?}", content, embedding);
        transaction.execute(&query, &[&content, &embedding])?;
    }

    info!("Data inserted");
    transaction.commit()?;
    Ok(())
}
