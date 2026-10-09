#!/usr/bin/env python3
"""Matched isolated release-process measurements. Only terminates its own children."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def run(*command):
    return subprocess.check_output(command, text=True, stderr=subprocess.STDOUT)


def cpu_seconds(text):
    # ps TIME uses [[days-]hours:]minutes:seconds.fraction.
    total = 0.0
    for component in text.strip().split(":"):
        total = total * 60 + float(component)
    return total


def footprint(text, peak=False):
    label = r"Physical footprint \(peak\)" if peak else "Physical footprint"
    match = re.search(label + r":\s*([\d.]+)([KMG])", text)
    if not match:
        raise ValueError("vmmap did not report a physical footprint")
    return float(match[1]) * {"K": 1 / 1024, "M": 1, "G": 1024}[match[2]]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--rounds", type=int, default=2)
    parser.add_argument("--interval", type=float, default=5)
    parser.add_argument("--settle", type=float, default=5)
    parser.add_argument("--out", type=Path, default=ROOT / "dist/benchmarks")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("vmmap measurements require macOS")
    if min(args.samples, args.rounds, args.interval, args.settle) <= 0:
        parser.error("measurement values must be positive")
    args.out.mkdir(parents=True, exist_ok=True)
    variants = {
        "egui": ROOT / "dist/bench-egui",
        "slint": ROOT / "target/release/rusty-pomodoro-slint",
    }
    report = {
        "date": datetime.datetime.now().astimezone().isoformat(),
        "host": run("sw_vers").strip(),
        "protocol": vars(args) | {"out": str(args.out)},
        "notes": [
            "No foreground activation requested; windows may be obscured by user's current app.",
            "Mouse/menu/hotkey input isolated while timing; scenario assertions stay enabled.",
            "Warmed filesystem/library caches, serial fresh processes, factory settings and empty history.",
            "Slint/egui use their current window defaults; native measurements are historical only.",
            "CPU percent calculated from ps accumulated-time delta over actual wall interval, not lifetime %cpu.",
            "vmmap physical footprint and RSS are different accounting metrics; peaks are vmmap-reported lifetime peaks.",
            "Statistics scenario opens statistics, not settings, at startup.",
            "Existing user processes and data are never terminated or read.",
        ],
        "bytes": {name: path.stat().st_size for name, path in variants.items()},
        "sha256": {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in variants.items()},
        "runs": [],
    }
    for iteration in range(args.rounds):
        # Rotate to reduce stable order bias across rounds.
        names = list(variants)
        names = names[iteration:] + names[:iteration]
        for name in names:
            for scenario in ("idle", "running", "statistics"):
                with tempfile.TemporaryDirectory(prefix=f"rusty-pomodoro-{name}-") as directory:
                    env = os.environ | {
                        "RUSTY_POMODORO_CONFIG_DIR": directory,
                        "RUSTY_POMODORO_BENCHMARK": "1",
                        "RUSTY_POMODORO_BENCHMARK_SCENARIO": scenario,
                    }
                    log_path = args.out / f"{name}-{scenario}-{iteration}.log"
                    with log_path.open("w") as log:
                        process = subprocess.Popen([str(variants[name])], env=env, stdout=log, stderr=log)
                        try:
                            time.sleep(args.settle)
                            if process.poll() is not None:
                                raise RuntimeError(f"{name} exited; inspect {log_path}")
                            previous_time = time.monotonic()
                            previous_cpu = cpu_seconds(run("ps", "-p", str(process.pid), "-o", "time="))
                            samples = []
                            for index in range(args.samples):
                                time.sleep(args.interval)
                                text = run("vmmap", "-summary", str(process.pid))
                                (args.out / f"{name}-{scenario}-{iteration}-{index}.vmmap.txt").write_text(text)
                                rss, accumulated = run("ps", "-p", str(process.pid), "-o", "rss=", "-o", "time=").split()
                                current_time = time.monotonic()
                                current_cpu = cpu_seconds(accumulated)
                                samples.append({
                                    "footprint_mib": footprint(text),
                                    "peak_mib": footprint(text, True),
                                    "rss_mib": int(rss) / 1024,
                                    "cpu_percent": (current_cpu - previous_cpu) / (current_time - previous_time) * 100,
                                    "elapsed_seconds": current_time - previous_time,
                                })
                                previous_time, previous_cpu = current_time, current_cpu
                            item = {
                                "variant": name, "scenario": scenario, "round": iteration, "pid": process.pid,
                                "samples": samples,
                            }
                            report["runs"].append(item)
                            print(name, scenario, iteration, json.dumps(samples), flush=True)
                        finally:
                            process.terminate()
                            try:
                                process.wait(timeout=5)
                            except subprocess.TimeoutExpired:
                                process.kill()
                                process.wait()
    report["summary"] = {}
    for name in variants:
        report["summary"][name] = {}
        for scenario in ("idle", "running", "statistics"):
            samples = [sample for item in report["runs"] if item["variant"] == name and item["scenario"] == scenario for sample in item["samples"]]
            report["summary"][name][scenario] = {
                metric: statistics.median(sample[metric] for sample in samples)
                for metric in ("footprint_mib", "rss_mib", "cpu_percent")
            } | {"peak_mib": max(sample["peak_mib"] for sample in samples)}
    (args.out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report["summary"], indent=2))


if __name__ == "__main__":
    main()
