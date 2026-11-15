#!/usr/bin/env python3
"""Aggregate visual_check reports for controlled baseline-versus-skill comparisons."""
from __future__ import annotations

import argparse
import json
import statistics
from collections import defaultdict
from pathlib import Path


def aggregate(directory: Path) -> dict:
    records = []
    for path in directory.rglob("report.json"):
        report = json.loads(path.read_text(encoding="utf-8"))
        if report.get("schema") != "egui-visual-forge.report.v1":
            continue
        if not report.get("tag") or not report.get("case_id"):
            continue
        records.append(report)
    if not records:
        raise ValueError("no tagged reports found (use visual_check --tag and --case-id)")
    tags = sorted({r["tag"] for r in records})
    if len(tags) < 2:
        raise ValueError("evaluation requires at least two conditions (different --tag values)")
    if any(not r.get("scene_quality_sha256") for r in records):
        raise ValueError("all reports must include scene_quality_sha256 from the current comparator")
    cases_by_tag = {tag: {r["case_id"] for r in records if r["tag"] == tag} for tag in tags}
    common = set.intersection(*cases_by_tag.values())
    if not common:
        raise ValueError("experiment tags have no shared case_id; comparison would be misleading")
    grouped = defaultdict(list)
    for record in records:
        if record["case_id"] in common:
            grouped[(record["tag"], record["case_id"])].append(record)
    # An equal number of image samples per case is preferable, but per-case means prevent
    # cases with many retries from dominating the overall comparison.
    case_stats = {}
    summary = {}
    for tag in tags:
        stat_list = []
        for case in sorted(common):
            runs = grouped[(tag, case)]
            # Check reference identity and physical dimensions across experiment conditions.
            hashes = {r["reference_sha256"] for r in runs}
            sizes = {tuple(r["pixel_size"]) for r in runs}
            states = {r["reference_state"] for r in runs}
            contracts = {r["scene_quality_sha256"] for r in runs}
            if len(hashes) != 1 or len(sizes) != 1 or len(states) != 1 or len(contracts) != 1:
                raise ValueError(f"inconsistent reference/quality contract within tag {tag}, case {case}")
            values = {
                "mae": statistics.mean(r["global"]["mae_rgb"] for r in runs),
                "changed_fraction": statistics.mean(r["global"]["changed_fraction"] for r in runs),
                "edge_f1": statistics.mean(r["global"]["edge_f1"] for r in runs),
                "pass_rate": statistics.mean(int(r["passed"]) for r in runs),
                "runs": len(runs),
                "reference_sha256": next(iter(hashes)),
                "pixel_size": next(iter(sizes)),
                "state": next(iter(states)),
                "quality_contract": next(iter(contracts)),
            }
            stat_list.append(values)
            case_stats[f"{tag}/{case}"] = values
        summary[tag] = {
            "case_count": len(common),
            "run_count": sum(x["runs"] for x in stat_list),
            "mean_case_mae": round(statistics.mean(x["mae"] for x in stat_list), 4),
            "mean_case_changed_fraction": round(statistics.mean(x["changed_fraction"] for x in stat_list), 6),
            "mean_case_edge_f1": round(statistics.mean(x["edge_f1"] for x in stat_list), 6),
            "mean_case_pass_rate": round(statistics.mean(x["pass_rate"] for x in stat_list), 6),
        }
    for case in common:
        sources = {case_stats[f"{tag}/{case}"]["reference_sha256"] for tag in tags}
        sizes = {tuple(case_stats[f"{tag}/{case}"]["pixel_size"]) for tag in tags}
        states = {case_stats[f"{tag}/{case}"]["state"] for tag in tags}
        contracts = {case_stats[f"{tag}/{case}"]["quality_contract"] for tag in tags}
        if len(sources) != 1 or len(sizes) != 1 or len(states) != 1 or len(contracts) != 1:
            raise ValueError(f"case {case} differs across conditions; cannot compare fairly")
    return {"schema": "egui-visual-forge.eval.v1", "case_ids": sorted(common),
            "excluded_unpaired_cases": sorted(set.union(*cases_by_tag.values()) - common),
            "condition_summary": summary, "case_details": case_stats,
            "caution": "Descriptive scores only. Must also control model, attempts, time, platform and app state."}


def markdown(results: dict) -> str:
    lines = ["# Controlled visual evaluation", "", f"Matched cases: {len(results['case_ids'])}", "",
             "| Condition | Cases | Runs | Mean MAE ↓ | Changed pixels ↓ | Edge F1 ↑ | Pass rate ↑ |",
             "|---|---:|---:|---:|---:|---:|---:|"]
    for tag, item in results["condition_summary"].items():
        lines.append(f"| {tag} | {item['case_count']} | {item['run_count']} | {item['mean_case_mae']:.2f} | {item['mean_case_changed_fraction']:.2%} | {item['mean_case_edge_f1']:.3f} | {item['mean_case_pass_rate']:.1%} |")
    lines += ["", f"Excluded unpaired cases: {results['excluded_unpaired_cases']}", "",
              "These are descriptive comparisons, not confidence intervals or proof of causality. Validate functional behavior separately.", ""]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = aggregate(args.reports)
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / "evaluation.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        (args.out / "evaluation.md").write_text(markdown(result), encoding="utf-8")
    except (ValueError, OSError, json.JSONDecodeError) as exc:
        parser.error(str(exc))
    print(f"Matched {len(result['case_ids'])} cases across {len(result['condition_summary'])} conditions; wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
