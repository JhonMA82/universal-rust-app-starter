# Installed skills

Canonical skills live in `.agents/skills`; `.claude/skills` is a relative symlink to that directory for clients that expect that path. Do not replace it with a copy. `scripts/check_agents.py` enforces the link and recreates it after generation when cargo-generate omits symlinks.

Installed (14): `idiomatic-rust`, `ownership-not-clone`, `rust-errors`, `rust-api-design`, `rust-testing`, `rust-code-review`, `rust-supply-chain`, `rust-docs`, `egui-visual-validation`, `egui-reconstruction`, `egui-scene-analysis`, `egui-ui-engineering`, `egui-runtime-verification`, `egui-visual-optimization`.

The eight Rust skills and `egui-visual-validation` are locally authored starter skills, **informed by** (not verbatim copies of) `rewrite-rs/skills` (BSD-3-Clause) and `full-stack-skills/rust-skills` (Apache-2.0). The five `egui-*` reconstruction skills are vendored from **egui-visual-forge v1.1** (MIT; see `LICENSE-EGUI-VISUAL-FORGE` and `docs/EGUI_VISUAL_FORGE.md`). Review upstream references before extending or importing additional third-party code. Skills run no network operations by themselves.

## Egui Visual Forge

The reconstruction skills use the bundled Python tooling. Optional setup and checks:

```sh
python3 -m pip install -r requirements.txt
python3 tools/validate_toolkit.py
python3 -m unittest discover -s tests -v
```

Start with `egui-reconstruction`; it delegates to the scene-analysis, UI-engineering, runtime-verification and optimization phases. The profile pins eframe/egui 0.36 while the forge example targets 0.37; match the workspace version before applying version-specific APIs.
