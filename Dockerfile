# Build stage
FROM rust:1.78 as builder
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
# Ensure logs go to stdout (for Docker logging)
ENV LOG_TO_STDOUT=true
# Configure logs to be written to file as well
ENV LOG_FILE_PATH=/app/logs/bot.log

# Run the binary
CMD ["./Blame_Serena"]
