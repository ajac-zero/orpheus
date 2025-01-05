benchmark:
  @uvx ai-mock server >/dev/null 2>&1 &
  @sleep 3
  cd scripts/benchmarks && hyperfine \
    --warmup 10 \
    --runs 50 \
    --command-name "openai" \
    "python _openai.py" \
    --command-name "langchain" \
    "python _langchain.py" \
    --command-name "orpheus" \
    "python _orpheus.py"
  @kill $(lsof -t -i:8100)
