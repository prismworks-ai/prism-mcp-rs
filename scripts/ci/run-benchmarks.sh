#!/usr/bin/env bash

set -euo pipefail

echo "Running maintained Criterion benchmark suite"
cargo bench --features bench,plugin,http --bench all_benchmarks

python3 <<'PYTHON'
import datetime
import json
import pathlib
import platform
import subprocess

root = pathlib.Path("target/criterion")
rows = []
for estimate_path in root.glob("**/base/estimates.json"):
    data = json.loads(estimate_path.read_text(encoding="utf-8"))
    estimate = data.get("median") or data.get("mean")
    if not estimate or "point_estimate" not in estimate:
        continue
    name = estimate_path.relative_to(root).as_posix().removesuffix("/base/estimates.json")
    rows.append((name, float(estimate["point_estimate"])))

if not rows:
    raise SystemExit("Criterion produced no parseable estimates; report not generated")

def format_ns(value):
    if value < 1_000:
        return f"{value:.2f} ns"
    if value < 1_000_000:
        return f"{value / 1_000:.3f} µs"
    if value < 1_000_000_000:
        return f"{value / 1_000_000:.3f} ms"
    return f"{value / 1_000_000_000:.3f} s"

def command_version(command):
    return subprocess.check_output(command, text=True).strip()

lines = [
    "# Benchmark Report",
    "",
    f"Generated: {datetime.datetime.now(datetime.timezone.utc).isoformat()}",
    "",
    "## Criterion results",
    "",
    "| Benchmark | Median estimate |",
    "|-----------|-----------------|",
]
for name, estimate in sorted(rows):
    lines.append(f"| `{name}` | {format_ns(estimate)} |")

lines.extend([
    "",
    "These are development/CI measurements, not production latency or throughput guarantees.",
    "",
    "## Environment",
    "",
    f"- OS: {platform.platform()}",
    f"- Architecture: {platform.machine()}",
    f"- Rust: {command_version(['rustc', '--version'])}",
    f"- Cargo: {command_version(['cargo', '--version'])}",
    "- Command: `cargo bench --features bench,plugin,http --bench all_benchmarks`",
    "",
    "Results vary with hardware, system load, compiler, features, and code. Use a stable runner and raw Criterion data for regression decisions.",
    "",
])

pathlib.Path("reports").mkdir(exist_ok=True)
pathlib.Path("reports/benchmark-report.md").write_text("\n".join(lines), encoding="utf-8")
PYTHON

echo "Benchmark report written to reports/benchmark-report.md"
