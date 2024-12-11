use embedding::vector_embedding::EmbedResponse;
use log::{debug, error, info};
use std::thread;
use tokio::task;

mod embedding;
mod vectordb;

#[tokio::main]
async fn main() {
    colog::init();
    info!("Starting");

    let url = "http://0.0.0.0:11434/api/embed";
    let model = "nomic-embed-text";
    // let input = vec!["hello".to_string()];
    let input: Vec<String> = vec![
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ]
    .iter()
    .map(|&s| s.to_string())
    .collect();

    let data = embedding::vector_embedding::EmbedRequest {
        model: model.to_string(),
        input: input,
    };

    let input_clone = data.input.clone();

    let embedding = task::spawn(async move {
        match embedding::vector_embedding::create_embed_request(url, &data).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error: {}", e);
                return EmbedResponse {
                    model: "".to_string(),
                    embeddings: vec![],
                };
            }
        }
    });

    let response = embedding.await.unwrap_or_else(|e| {
        error!("Error: {:?}", e);
        EmbedResponse {
            model: "".to_string(),
            embeddings: vec![],
        }
    });

    // query the embeddings
    let query_input = vec!["some animal is purring".to_string()];
    let query_data = embedding::vector_embedding::EmbedRequest {
        model: model.to_string(),
        input: query_input,
    };

    let query_embedding = task::spawn(async move {
        match embedding::vector_embedding::create_embed_request(url, &query_data).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error: {}", e);
                return EmbedResponse {
                    model: "".to_string(),
                    embeddings: vec![],
                };
            }
        }
    });

    let query_response = query_embedding.await.unwrap_or_else(|e| {
        error!("Error: {:?}", e);
        EmbedResponse {
            model: "".to_string(),
            embeddings: vec![],
        }
    });

    // create new thread to embed data
    let embed_thread = thread::spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client() {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        let table = "from_rust";
        let dim = 768;
        match vectordb::pg_vector::create_table(&mut client, table, dim) {
            Ok(_) => {
                info!("Create table successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        match vectordb::pg_vector::load_vector_data(
            &mut client,
            table,
            &input_clone,
            &response.embeddings,
        ) {
            Ok(_) => {
                info!("Load vector data successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        // match vectordb::pg_vector::select_embeddings(&mut client, &table) {
        //     Ok(_) => {
        //         info!("Select main successful");
        //     }
        //     Err(e) => {
        //         error!("Error: {}", e);
        //         return;
        //     }
        // };

        // query the embeddings
        let query =
            vectordb::pg_vector::query_nearest(&mut client, table, &query_response.embeddings);
        match query {
            Ok(_) => {
                debug!("Query nearest vector successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    if let Err(e) = embed_thread.join() {
        error!("Error: {:?}", e);
        return;
    }

    info!("Done with main");
}
