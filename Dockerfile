FROM rust:latest as builder


WORKDIR /app

# Copy files
COPY . .

# Build release binary
RUN cargo build --release

# runtime image
FROM debian:bookworm-slim

# Install runtime dependencies (e.g., for LanceDB or networking)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/app /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["app"]

# Default command (can be overridden)
CMD ["--help"]