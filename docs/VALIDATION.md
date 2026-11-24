# Verification and evidence

Run from the starter checkout with Python 3.11+, Rust, rustfmt, Clippy and Cargo
Generate 0.26.0. Install the pinned Python generator requirements.

```sh
python3 tools/verify_templates.py
python3 -m unittest discover -s tests -v
python3 tools/check_docs.py
python3 tools/verify_templates.py --generate --cargo-check
python3 tools/verify_composed.py --ordinary --evidence evidence/ordinary.json
python3 tools/verify_composed.py --evidence evidence/references.json
python3 tools/verify_composed.py --hybrid --evidence evidence/hybrid.json
```

Reference checks build the generated binaries and execute CLI and MCP
scenarios. Live MCP checks discovery, authentication, operation execution and
exit over local Streamable HTTP. Use `--visual` for wgpu gallery and viewport
captures. Use `--native-window` for an opt-in native inspection test from the
Document Workbench. It needs a real display and accesses the same inspection
endpoint used by the development bridge.

Run `python3 tools/verify_browser.py --screenshots evidence/browser` with
Trunk and Playwright Chromium installed. It exercises a Leptos operation and
verifies that the hybrid egui runner starts after loading its WASM bundle.
The workflow covers native tests on Linux, Windows and macOS, with separate
visual, browser and WASM jobs on Linux. Workflow status is independent from
locally produced evidence. Store evidence outside the source tree or in the
ignored `evidence/` directory.
