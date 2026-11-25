---
name: egui-visual-optimization
description: Measures and iteratively improves screenshot fidelity of an egui application using unaltered reference/candidate images, region-level metrics, diff heatmaps, causal hypotheses, and explicit acceptance gates. Use after each egui UI capture or when diagnosing mismatched pixels or layout.
license: MIT
compatibility: Python 3.11+ managed with uv, Pillow, NumPy; source and candidate PNG files.
metadata:
  version: "1.0.0"
---

# Visual QA as a controlled optimization loop

## Execute

1. Read scene metadata, verify source reference hash, state, crop, dimensions and scale. Comparison aborts on dimension mismatch or modified reference.
2. Run `uv run python tools/visual_check.py --scene scene.json --candidate candidate.png --out qa/iteration-01` from the project root after one `uv sync`.
3. Read `report.json` and `report.md`. Inspect `comparison.png` and `heatmap.png`. Treat the summary score as a diagnostic, not an aesthetic certificate.
4. Rank region failures by impact: major structural edges and missing text before fine borders, shadows and minor colors.
5. For the top defect, write a causal hypothesis: *observable error → likely layout/style/state cause → proposed minimal code change*.
6. Make one small family of connected changes. Re-run build/tests, capture a new candidate and compare with exactly the same manifest and thresholds.
7. Retain previous artifacts. If a correction improves one region but breaks another, diagnose/regress rather than quietly accepting it.
8. Stop only when project-specific gates pass AND semantic/function tests pass, or when a documented blocker/iteration budget is reached. Never hide mismatch by raising thresholds, adding masks, or shifting the candidate.

## Metrics and interpretation

- `mae_rgb`: mean absolute 0–255 difference over RGB channels. A large flat background may dominate it.
- `changed_fraction`: fraction of included pixels whose **largest channel difference** exceeds the configured threshold.
- `edge_f1`: edge overlap with one-pixel tolerance to prioritize structural boundary alignment. It is not a semantic judgment.
- Per-region gates combine with global validation; ignored pixels are excluded, but mask size is bounded.
- `--diagnose-shift 4` reports the best small candidate translation for diagnosis **without applying any alignment to the official metrics**.

The metrics are approximate and susceptible to font rasterization, color management, and GPU differences. They only compare two renderings with the same canvas and state.

## Required failure categorization

Use: `capture/state`, `macro-layout`, `text/typography`, `widget dimensions`, `palette`, `icons/assets`, `rendering-platform`, `unknown`. Diagnose P0 before P2. Use `references/triage.md`.

## Command and report

```bash
uv run python tools/visual_check.py --scene scene.json --candidate candidate.png \
  --out qa/iteration-01 --tag with-skills --case-id dashboard-01 --run-id 01 \
  --diagnose-shift 4
```

Exit codes: 0 passes all configured gates, 1 invalid input/runtime error, 3 comparison completed but failed quality gates. For unbiased benchmarking, preserve individual reports; aggregate them with `tools/evaluate_runs.py`.
