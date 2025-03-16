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
# Copy static files for web server
COPY --from=builder /usr/src/app/static /app/static

# Create directory for logs
RUN mkdir -p /app/logs

# Set environment variables
ENV RUST_LOG=info
# Mark as Docker environment for logging configuration
ENV DOCKER_ENVIRONMENT=true
# Configure logging to go directly to stdout in Docker
ENV LOG_TO_STDOUT=true
# Keep log file path but it will be ignored in Docker mode
ENV LOG_FILE_PATH=/app/logs/bot.log

# Simple script to handle signals properly
RUN echo '#!/bin/bash\n\
# Forward signals to the application\n\
trap "kill -TERM \$child" SIGTERM SIGINT\n\
\n\
# Start the application\n\
./Blame_Serena & \n\
child=$!\n\
\n\
# Wait for the application to terminate\n\
wait "$child"\n\
\n\
echo "Bot application has exited with status $?"\n\
' > /app/run.sh && chmod +x /app/run.sh

# Run the application with proper signal forwarding
CMD ["/app/run.sh"]
