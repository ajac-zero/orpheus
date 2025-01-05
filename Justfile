benchmark:
  @uvx ai-mock server >/dev/null 2>&1 &
  @sleep 3
  cd scripts/benchmarks && hyperfine \
    --warmup 10 \
    --runs 50 \
    "python _openai.py" \
    "python _langchain.py" \
    "python _orpheus.py"
  @kill $(lsof -t -i:8100)
