# Verification and evidence

Run from the starter checkout with Python 3.11+, Rust, rustfmt, Clippy and Cargo
Generate 0.26.0. Install the pinned Python generator requirements.

```sh
python3 tools/verify_templates.py
python3 -m unittest discover -s tests -v
python3 tools/check_docs.py
python3 tools/verify_templates.py --generate --cargo-check
python3 tools/verify_composed.py --ordinary --evidence evidence/ordinary.json
python3 tools/verify_domain_scenarios.py --evidence evidence/domain-scenarios.json
python3 tools/verify_composed.py --evidence evidence/references.json
python3 tools/verify_composed.py --hybrid --evidence evidence/hybrid.json
```

The Domain-first scenarios exercise six generated compositions. The combined
scenario tests one document and its derived data and geometry in the tank
workspace. It drives a background calculation through egui's accessibility
harness and verifies tank edits, queries and undo against the generated MCP
server. The evidence records the work performed
in each generated workspace.

The ordinary matrix generates and checks Minimal and Application for all
eleven profiles, then exercises the native Workbench and Engineering presets.
Reference checks build the generated binaries and execute CLI and MCP scenarios. Live MCP checks discovery, authentication, operation execution and
exit over local Streamable HTTP. Use `--visual` for wgpu gallery and viewport
captures. Use `--native-window` for an opt-in native inspection test from the
Document Workbench. The Visual Forge tooling is a uv project in
`templates/desktop-egui`: run `uv sync --project templates/desktop-egui` once,
then run the native verifier with
`uv run --project templates/desktop-egui --with-requirements generator/requirements.txt python tools/verify_composed.py ...`
(the overlay adds the generator's `tomli-w` to the Visual Forge environment). On Linux, the
window needs a display and the X11 keyboard runtime package
`libxkbcommon-x11-0`. A 96-DPI Xvfb run supplies a virtual display. The
test uses the development inspection endpoint to find and click a widget,
checks the updated AccessKit tree, and compares captures with Visual Forge.

Run `python3 tools/verify_browser.py --screenshots evidence/browser` with
Trunk, Dioxus CLI 0.7.10, Pillow and Playwright Chromium installed. Leptos and Dioxus
rename a workspace using the rendered controls. The hybrid app creates two
notes by keyboard and verifies each announcement before capturing the canvas.
The verifier rejects a blank capture even when interaction passes. Use `--profile` to retry one case and `--chromium` to use an installed
Chromium executable.
The workflow covers native tests on Linux, Windows and macOS, with separate
visual, browser and WASM jobs on Linux. Workflow status is independent from
locally produced evidence. Store evidence outside the source tree or in the
ignored `evidence/` directory.
