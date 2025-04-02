#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chat::run_chat;
    use configs::constants::{AI_MODEL, CHAT_API_KEY, CHAT_API_URL};
    use mockito::Server;

    #[tokio::test]
    async fn test_run_chat_success() -> Result<()> {
        // Create the mock server first OUTSIDE the async context
        let mut server = Server::new_async().await;

        let mock_response = r#"
        {
            "message": {
                "role": "assistant",
                "content": "Hello, how can I help you?"
            }
        }"#;

        server
            .mock("POST", "")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = configs::get_https_client().unwrap();

        // Define parameters
        let system_prompt = "tests/resources/rag_prompt.txt";
        let ai_prompt = "Hello!";
        let provider = "ollama";
        let api_url = CHAT_API_URL;
        let api_key = CHAT_API_KEY;
        let ai_model = AI_MODEL;

        // Call your async function
        let response = run_chat(
            system_prompt,
            ai_prompt,
            None,
            &client,
            provider,
            api_url,
            api_key,
            ai_model,
        )
        .await
        .unwrap();

        // Verify the response
        assert!(response.get_message().is_some());

        Ok(())
    }
}
