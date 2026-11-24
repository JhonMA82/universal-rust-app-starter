# {{project-name}}

Generated with **Universal Rust App Starter** — profile `desktop-egui`.

Native egui/eframe application.

## Requirements

- Rust stable (`rustup`, `cargo`, Rust 2024 edition)
- Python 3.11+ for the offline agent-readiness check
- Optional: Pillow, NumPy and PyYAML for the Egui Visual Forge screenshot tooling
- Profile-specific tooling only when relevant (see below)

## Quick start

```sh
cargo check --workspace --all-targets
cargo test --workspace
python3 scripts/check_agents.py
```

See `docs/ARCHITECTURE.md`, `AGENTS.md`, and `docs/SKILLS.md`. CI runs in GitHub Actions and includes dependency advisory checks on a weekly schedule. Replace copyright holders in `LICENSE-MIT` and add a repository URL before distribution. For an application, generate and commit `Cargo.lock` before release (`cargo generate-lockfile`).

## Profile-specific startup

See below; adapt the starter example rather than building your production domain into it.

```sh
cargo run -p desktop
cargo test --workspace
```
Prefer modifying `crates/app` rather than platform runners for shared UI behavior. Linux needs Wayland or X11 libraries.

## Egui Visual Forge

The `egui-*` reconstruction skills drive a screenshot-to-egui workflow with bundled comparison tools:

```sh
python3 -m pip install -r requirements.txt
python3 tools/validate_toolkit.py
python3 -m unittest discover -s tests -v
```

Start with `.agents/skills/egui-reconstruction/SKILL.md` and read `docs/EGUI_VISUAL_FORGE.md`. The example under `examples/eframe-starter` is standalone and targets eframe 0.37; this workspace uses the same version. Python tooling is optional for normal builds.
