#!/bin/bash

# Set variables
ORPHEUS_API_KEY="mock"
ORPHEUS_BASE_URL="http://localhost:8100/openai"

# Start the mock server
uvx ai-mock server >/dev/null 2>&1 &
MOCK_SERVER_PID=$! # Get the PID of the background process
sleep 3

# Run hyperfine
hyperfine \
  --warmup 10 \
  --runs 50 \
  "python _openai.py" \
  "python _langchain.py" \
  "python _orpheus.py"

# Kill the mock server
kill "$MOCK_SERVER_PID"

exit 0
