#[cfg(test)]
mod tests {
    use embedder::file_loader::chunk_embed_request;
    use embedder::file_loader::is_supported_file;
    use embedder::file_loader::FileChunk;
    use embedder::file_loader::Language;
    use std::path::PathBuf;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("rs"), Language::Rust);
        assert_eq!(Language::from_str("py"), Language::Python);
        assert_eq!(Language::from_str("unknown"), Language::UNKNOWN);
        assert_eq!(Language::from_str("nonexistent"), Language::UNKNOWN);
    }

    #[test]
    fn test_file_chunk_creation() {
        let content = "line 1\nline 2\nline 3".to_string();
        let file_path = PathBuf::from("test.rs");
        let chunk_number = 1;

        let chunk = FileChunk::new(content.clone(), file_path.clone(), chunk_number);

        assert_eq!(chunk.get_content(), content);
        assert_eq!(chunk.get_file_path(), &file_path);
        assert_eq!(chunk.get_chunk_number(), chunk_number);
    }

    #[test]
    fn test_chunk_embed_request() {
        let content = "some content".to_string();
        let file_path = PathBuf::from("test.py");
        let chunk_number = 0;
        let chunk = FileChunk::new(content, file_path, chunk_number);

        let provider = "test_provider";
        let api_url = "http://test.api";
        let api_key = "test_key";
        let model = "test_model";

        let embed_request = chunk_embed_request(&chunk, provider, api_url, api_key, model);

        assert_eq!(embed_request.provider, provider);
        assert_eq!(embed_request.api_url, api_url);
        assert_eq!(embed_request.api_key, api_key);
        assert_eq!(embed_request.model, model);
        assert_eq!(embed_request.input, chunk.content);
        assert_eq!(embed_request.metadata, Some("test.py".to_string()));
        assert_eq!(embed_request.chunk_number, Some(chunk_number));
    }

    #[test]
    fn test_is_supported_file() {
        let supported_path = std::path::Path::new("test.rs");
        let unsupported_path = std::path::Path::new("unknown.xyz");

        let (language, is_supported) = is_supported_file(supported_path);
        assert_eq!(language, Language::Rust);
        assert!(is_supported);

        let (language, is_supported) = is_supported_file(unsupported_path);
        assert_eq!(language, Language::UNKNOWN);
        assert!(!is_supported);
    }

    // @TODO tests for other functions .
}
