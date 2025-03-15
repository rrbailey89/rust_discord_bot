# Build stage
FROM rust:1.76 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/Blame_Serena /app/
# Copy migrations directory
COPY --from=builder /usr/src/app/migrations /app/migrations

# Create directory for logs
RUN mkdir -p /app/logs

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["./Blame_Serena"]
