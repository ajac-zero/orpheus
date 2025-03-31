# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "pyperf",
#     "langchain-openai",
#     "litellm",
#     "openai",
#     "rich",
#     "matplotlib",
#     "argparse",
#     "orpheus",
# ]
#
# [tool.uv.sources]
# orpheus = { path = "../", editable = true }
#
# [tool.ruff]
# line-length = 120
# ///
import argparse
import functools
import statistics
import time

import matplotlib.pyplot as plt
from langchain_openai import ChatOpenAI
from litellm import acompletion
from openai import AsyncOpenAI
from rich.console import Console
from rich.progress import track
from rich.table import Table

from orpheus import AsyncOrpheus

# Constants

API_KEY = "mock"
BASE_URL = "http://localhost:8100/openai"
MODEL = "gpt-4o"
MESSAGES = [
    {"role": "system", "content": "You are a friendly bot"},
    {
        "role": "user",
        "content": [
            {"type": "text", "text": "hello, whats in the image"},
            {
                "type": "image_url",
                "image_url": {
                    "url": "https://www.pawlovetreats.com/cdn/shop/articles/pembroke-welsh-corgi-puppy_2000x.jpg?v=1628638716"
                },
            },
        ],
    },
]


class Runner:
    def __init__(
        self,
        console: Console,
        targets: list,
        runs: int,
        variance: bool = False,
        save: bool = False,
    ):
        self.targets = targets
        self.console = console
        self.runs = range(runs)
        self.variance = variance
        self.save = save

    async def evaluate(self):
        results = [await self.time_function(func, name) for name, func in self.targets]

        # Print and visualize results
        self.print_results(results)

        try:
            self.plot_results(results)
        except Exception as e:
            self.console.print(f"Could not generate plot: {str(e)}", style="red")

        # Provide analysis
        self.analyze_results(results)

        return results

    async def time_function(self, func, name):
        """Time a function's execution over multiple runs"""
        self.console.print(f"Running [bold]{name}[/bold] benchmark...", style="blue")
        times = []
        results = []

        for i in track(self.runs, description=""):
            try:
                start_time = time.time()
                result = await func()
                end_time = time.time()

                execution_time = end_time - start_time
                times.append(execution_time)
                results.append(result)

            except Exception as e:
                self.console.print(f"  Run {i + 1}: Failed - {str(e)}", style="red")

        return {
            "name": name,
            "times": times,
            "results": results,
            "mean": statistics.mean(times) if times else None,
            "median": statistics.median(times) if times else None,
            "min": min(times) if times else None,
            "max": max(times) if times else None,
            "stdev": statistics.stdev(times) if len(times) > 1 else None,
        }

    def print_results(self, benchmark_results):
        """Print benchmark results in a table"""
        table = Table(title="SDK Performance Benchmark")

        table.add_column("Client", style="cyan", no_wrap=True)
        table.add_column("Mean (s)", style="magenta")
        table.add_column("Median (s)", style="magenta")
        table.add_column("Min (s)", style="green")
        table.add_column("Max (s)", style="red")
        table.add_column("Std Dev", style="yellow")

        for result in benchmark_results:
            if result["mean"] is not None:
                table.add_row(
                    result["name"],
                    f"{result['mean']:.4f}",
                    f"{result['median']:.4f}",
                    f"{result['min']:.4f}",
                    f"{result['max']:.4f}",
                    f"{result['stdev']:.4f}" if result["stdev"] is not None else "N/A",
                )
            else:
                table.add_row(
                    result["name"], "Failed", "Failed", "Failed", "Failed", "Failed"
                )

        self.console.print()
        self.console.print(table)

    def plot_results(self, benchmark_results):
        """Create a bar chart of the mean times"""
        names = [r["name"] for r in benchmark_results]
        means = [r["mean"] for r in benchmark_results]

        plt.figure(figsize=(10, 6))
        bars = plt.bar(names, means, color=["#1f77b4", "#ff7f0e", "#2ca02c"])

        # Add labels and values on top of bars
        for bar in bars:
            height = bar.get_height()
            plt.text(
                bar.get_x() + bar.get_width() / 2.0,
                height + 0.01,
                f"{height:.4f}s",
                ha="center",
                va="bottom",
            )

        plt.ylabel("Mean Execution Time (seconds)")
        plt.title("SDK Performance Comparison")
        plt.grid(axis="y", linestyle="--", alpha=0.7)

        # Save the figure
        if self.save:
            plt.savefig("sdk_benchmark_results.png")
            self.console.print(
                "Benchmark chart saved as [bold]api_benchmark_results.png[/bold]",
                style="green",
            )

    def analyze_results(self, benchmark_results):
        """Provide some analysis of the results"""
        self.console.print("\n[bold]Analysis:[/bold]", style="yellow")

        if not all(r["mean"] for r in benchmark_results):
            return "Some benchmarks failed to complete"

        # Sort by mean execution time
        sorted_results = sorted(benchmark_results, key=lambda x: x["mean"])
        fastest = sorted_results[0]
        slowest = sorted_results[-1]

        # Display fastest and slowest results
        analysis = [
            f"The fastest client was [cyan]{fastest['name']}[/] with a mean time of {fastest['mean']:.4f} seconds.",
            f"The slowest client was [cyan]{slowest['name']}[/] with a mean time of {slowest['mean']:.4f} seconds.\n",
        ]

        # Calculate percentage differences
        for candidate in sorted_results[1:]:
            speedup = (candidate["mean"] - fastest["mean"]) / candidate["mean"] * 100

            analysis.append(
                f"Using [cyan]{fastest['name']}[/] instead of [cyan]{candidate['name']}[/] provides a [cyan]{speedup:.1f}%[/] speed improvement."
            )

        if self.variance:
            # Compare consistency (standard deviation as percentage of mean)
            for result in benchmark_results:
                if result["stdev"] is not None and result["mean"] > 0:
                    variation = (result["stdev"] / result["mean"]) * 100
                    consistency = (
                        "highly consistent"
                        if variation < 5
                        else "somewhat variable"
                        if variation < 15
                        else "highly variable"
                    )
                    analysis.append(
                        f"{result['name']} was {consistency} with {variation:.1f}% variation across runs."
                    )

        self.console.print("\n".join(analysis))


if __name__ == "__main__":
    import asyncio

    # Define program arguments
    parser = argparse.ArgumentParser(
        prog="SDK Benchmark",
        description="Compare performance across OpenAI, LangChain, LiteLLM, and Orpheus",
        epilog="Made with love <3",
    )
    parser.add_argument("-r", "--runs", default=3000, type=int, help="Number of runs")
    parser.add_argument("-v", "--variance", action="store_true", help="Show variance")
    parser.add_argument("-s", "--save", action="store_true", help="Save plot")

    # Parse arguments if any
    args = parser.parse_args()

    # Initialize clients
    openai = AsyncOpenAI(api_key=API_KEY, base_url=BASE_URL)
    langchain = ChatOpenAI(api_key=API_KEY, base_url=BASE_URL)
    orpheus = AsyncOrpheus(api_key=API_KEY, base_url=BASE_URL)

    # Setup function calls
    openai_func = functools.partial(
        openai.chat.completions.create, messages=MESSAGES, model=MODEL
    )
    langchain_func = functools.partial(langchain.ainvoke, input=MESSAGES, model=MODEL)
    orpheus_func = functools.partial(
        orpheus.chat.completions.create, messages=MESSAGES, model=MODEL
    )
    litellm_func = functools.partial(
        acompletion, model=MODEL, messages=MESSAGES, api_key=API_KEY, base_url=BASE_URL
    )

    console = Console()

    # Print header
    console.print("[bold]Async SDK Benchmark[/bold]", style="yellow")
    console.print(
        f"Each sdk will be tested with [bold]{args.runs}[/bold] completion calls\n"
    )

    # Define benchmark targets
    targets = [
        ("OpenAI", openai_func),
        ("LangChain", langchain_func),
        ("LiteLLM", litellm_func),
        ("Orpheus", orpheus_func),
    ]

    # Run benchmark
    crt = Runner(console, targets, args.runs, args.variance, args.save).evaluate()
    asyncio.run(crt)
