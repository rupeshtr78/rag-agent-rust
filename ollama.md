To create an Ollama Docker setup that uses the `nomic-embed-text` model, you need to ensure that the Docker container can download and use the model, as well as any necessary configuration files. Here’s a step-by-step guide to achieve this:

### Create a `Modelfile`

First, create a `Modelfile` that specifies the base model and any additional parameters. Here is an example `Modelfile` for the `nomic-embed-text` model, although note that this model is specifically for generating embeddings and does not require the same parameters as a language model:

```plaintext
FROM nomic-embed-text
```

Since `nomic-embed-text` is an embedding model, it does not need additional parameters like `temperature` or `num_ctx`.

### Create a Shell Script

Create a shell script (`run_ollama.sh`) that will download the model, create the necessary configuration, and start the Ollama service:

```bash
#!/bin/bash

# Pull the nomic-embed-text model
ollama pull nomic-embed-text

# Start Ollama service
ollama serve &

# Keep the service running
tail -f /dev/null
```

### Dockerfile

Create a `Dockerfile` to copy the script and make it executable:

```dockerfile
FROM ollama/ollama:latest

COPY run_ollama.sh /run_ollama.sh
RUN chmod +x /run_ollama.sh
```

### Docker Compose File

Create a `docker-compose.yml` file to define the service and volumes:

```yaml
version: '3'

services:
  ollama:
    build: .
    container_name: ollama
    ports:
      - "11434:11434"
    volumes:
      - ./model_files:/model_files
      - ollama_volume:/root/.ollama
    command: /run_ollama.sh

volumes:
  ollama_volume:
```

### Directory Structure

Ensure your directory structure is as follows:
```
- model_files/
  - Modelfile
- run_ollama.sh
- docker-compose.yml
- Dockerfile
```

### Running the Setup

To start the Ollama service with the `nomic-embed-text` model, run:

```bash
docker-compose up -d
```

This setup will pull the `nomic-embed-text` model, start the Ollama service, and keep it running in the background.

### Using the Model

Once the service is running, you can use the `nomic-embed-text` model via the Ollama API. Here is an example of how to generate embeddings using `curl`:

```bash
curl http://localhost:11434/api/embeddings -d '{
  "model": "nomic-embed-text",
  "prompt": "The sky is blue because of Rayleigh scattering"
}'
```


`Dockerfile` for the repository



### Key Features:
1. **Multi-stage build**: 
   - Builder stage compiles the Rust project.
   - Final stage uses a minimal Debian image for runtime.

2. **Dependencies**:
   - Only includes `ca-certificates` for HTTPS support (adjust if additional runtime dependencies are needed for LanceDB/Ollama).

3. **Entrypoint**:
   - Defaults to running the binary with `--help`.

### Usage Notes:
1. **Build the image**:
   ```bash
   docker build -t rag-agent-rust .
   ```

2. **Run the container**:
   ```bash
   docker run --rm rag-agent-rust [COMMAND] [ARGS]
   # Example:
   docker run --rm rag-agent-rust load -p /data/sample/
   ```

3. **Volumes for data**:
   - Mount a volume for LanceDB persistence:
     ```bash
     docker run -v ./data:/data rag-agent-rust
     ```

4. **Ollama Integration**:
   - If Ollama runs locally, use `--network=host` to connect:
     ```bash
     docker run --network=host rag-agent-rust chat -p "query"
     ```

### Adjustments Needed:
- Add environment variables (e.g., `OLLAMA_HOST`) if required by the app.
- Include additional runtime dependencies in the final image if the project needs them (check `README.md` for prerequisites).