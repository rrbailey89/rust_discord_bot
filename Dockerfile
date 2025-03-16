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

# Script to run the binary and tail logs to stdout for Docker with log rotation support
RUN echo '#!/bin/bash\n\
# Start the application\n\
./Blame_Serena & \n\
BOT_PID=$! \n\
\n\
# Initial wait for log files to be created\n\
sleep 2\n\
\n\
# Function to find the most recent log file\n\
find_latest_log() {\n\
  ls -t /app/logs/bot.log* 2>/dev/null | head -n 1\n\
}\n\
\n\
# Function to handle cleanup on exit\n\
cleanup() {\n\
  echo "Cleaning up..."\n\
  [ -n "$TAIL_PID" ] && kill $TAIL_PID 2>/dev/null\n\
  exit 0\n\
}\n\
\n\
# Set trap for cleanup\n\
trap cleanup SIGTERM SIGINT\n\
\n\
# Main loop to continuously check for new log files\n\
while kill -0 $BOT_PID 2>/dev/null; do\n\
  CURRENT_LOG_FILE=$(find_latest_log)\n\
  \n\
  # If we do not have a log file yet, wait and try again\n\
  if [ -z "$CURRENT_LOG_FILE" ]; then\n\
    echo "Waiting for log file to be created..."\n\
    sleep 5\n\
    continue\n\
  fi\n\
  \n\
  echo "Tailing log file: $CURRENT_LOG_FILE"\n\
  tail -f "$CURRENT_LOG_FILE" & \n\
  TAIL_PID=$!\n\
  \n\
  # Check for new log files every minute\n\
  for i in {1..60}; do\n\
    sleep 1\n\
    # Check if the bot is still running\n\
    if ! kill -0 $BOT_PID 2>/dev/null; then\n\
      echo "Bot process has exited"\n\
      cleanup\n\
      break\n\
    fi\n\
  done\n\
  \n\
  # Check if a new log file was created\n\
  NEW_LOG_FILE=$(find_latest_log)\n\
  if [ "$NEW_LOG_FILE" != "$CURRENT_LOG_FILE" ]; then\n\
    echo "New log file detected: $NEW_LOG_FILE"\n\
    # Stop tailing the old file\n\
    kill $TAIL_PID 2>/dev/null\n\
  fi\n\
done\n\
\n\
# Clean up any remaining tail process\n\
[ -n "$TAIL_PID" ] && kill $TAIL_PID 2>/dev/null\n\
\n\
echo "Bot application has exited"\n\
' > /app/run.sh && chmod +x /app/run.sh

# Run the script that redirects logs
CMD ["/app/run.sh"]
