# Egui Visual Forge

Vendored from **egui-visual-forge v1.1** (MIT; see `LICENSE-EGUI-VISUAL-FORGE`). It adds a measurable screenshot-to-egui workflow on top of the existing `egui-visual-validation` skill.

## Skills

| Skill | Responsibility |
|---|---|
| `egui-reconstruction` | End-to-end orchestration and delivery criteria |
| `egui-scene-analysis` | Geometry, scale, text, tokens and uncertainties from a screenshot |
| `egui-ui-engineering` | Rust/egui implementation from a scene manifest |
| `egui-runtime-verification` | Deterministic captures, `egui_kittest`, optional AccessKit/`egui_mcp` |
| `egui-visual-optimization` | Region-level comparison and hypothesis-driven iteration |

Skill sources live in `.agents/skills/`; `.claude/skills/` is a byte-identical mirror. Only load the skill needed for the current phase.

## Bundled tooling

- `tools/init_scene.py`: creates `scene.json` with a SHA-256 reference fingerprint.
- `tools/visual_check.py`: compares equal-size PNGs without rescaling or aligning them; emits MAE, changed fraction, edge F1, per-region reports, heatmap, overlay and side-by-side comparison.
- `tools/evaluate_runs.py`: paired A/B aggregation that averages by case.
- `tools/validate_toolkit.py`: validates the forge skills, local references, Python syntax and required files.
- `docs/SCENE_FORMAT.md`: the `egui-visual-forge.scene.v1` contract.
- `docs/WORKFLOW.md`, `docs/RESEARCH.md`: operating procedure and design sources.
- `examples/eframe-starter/`: a standalone eframe 0.37 example with an `egui_kittest` interaction test.
- `tests/test_tools.py`: comparator, manifest, mask and evaluation tests.

## Setup

```sh
python3 -m pip install -r requirements.txt
python3 tools/validate_toolkit.py
python3 -m unittest discover -s tests -v
```

These checks are optional for normal builds and are not part of the Rust CI. They require Pillow, NumPy and PyYAML.

## Workflow

1. Keep the reference PNG unchanged; record its SHA-256 and `pixels_per_point`.
2. Create the manifest with `python3 tools/init_scene.py --image reference.png --out scene.json --pixels-per-point 1`, then replace placeholder measurements with evidence.
3. Implement the scene with real egui widgets and deterministic initial state.
4. Capture a candidate with the exact same window content, size, scale, theme and state.
5. Run `python3 tools/visual_check.py --scene scene.json --candidate candidate.png --out qa/iteration-01`.
6. Fix the highest-impact region, recapture and compare with unchanged masks and thresholds.

Do not alter the reference, resize either image, shift the candidate for the official score, or loosen gates to make a run pass.

## Version note

This profile pins `eframe`/`egui` **0.37** in the workspace `Cargo.toml`. The bundled example and the forge skills use **0.37** as their baseline. Confirm the pinned version's APIs before applying version-specific recipes, or upgrade the workspace deliberately and rerun the full checks. Do not mix APIs from different egui releases.

## Verification boundary

The toolkit's Python tests validate the comparator and manifest handling, not screenshot fidelity. A visual claim requires a real capture of the running application plus Rust build and interaction checks. See the generated `AGENTS.md` for the project checks.
