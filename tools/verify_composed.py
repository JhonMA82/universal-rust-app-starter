#!/usr/bin/env python3
"""Generate and verify the modular references and record observed results."""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from generator.compose import ROOT, generate, read_json

SCENARIOS = {
    "document-workbench": [
        ("notes.edit", {"text": "round trip"}, "1", {"text": "round trip"}),
        ("notes.undo", {}, "1", {"text": ""}),
        ("notes.redo", {}, "1", {"text": "round trip"}),
    ],
    "data-explorer": [
        ("data.import", {"csv": "name,value\nkeep,2\nexclude,100\nkeep,4\n"}, None, 3),
        ("data.filter", {"query": "keep", "column": 1, "sort": None}, None, 2),
        ("data.summary", {}, None, {"mean": 3}),
        ("data.csv", {}, None, "name,value\nkeep,2\nkeep,4\n"),
    ],
    "engineering-workbench": [
        ("box.resize", {"width_m": 2, "height_m": 3, "depth_m": 4}, None, {"width_m": 2}),
        ("box.volume", {}, None, 24),
        ("box.undo", {}, None, {"height_m": 1}),
        ("box.volume", {}, None, 4),
    ],
}


def expect(actual, expected):
    if isinstance(expected, dict):
        assert all(actual[key] == value for key, value in expected.items()), actual
    else:
        assert actual == expected, (actual, expected)


def cli(exe, project, scenario):
    op, values, document, expected = scenario[0]
    request = {"command_id": op, "parameters": values, "document_id": document, "session_id": "local", "expected_revision": 0}
    result = subprocess.run([str(exe), "--invoke", json.dumps(request)], cwd=project,
                            capture_output=True, text=True, check=True, timeout=15)
    expect(json.loads(result.stdout)["ok"]["result"], expected)


def mcp(exe, project, scenario):
    with subprocess.Popen([str(exe), "--mcp"], cwd=project, text=True, encoding="utf-8",
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE) as peer:
        messages = queue.Queue()
        def collect():
            for line in peer.stdout:
                messages.put(line)
        threading.Thread(target=collect, daemon=True).start()
        cursor = 0

        def send(message):
            peer.stdin.write(json.dumps({"jsonrpc": "2.0", **message}) + "\n")
            peer.stdin.flush()

        def call(method, params):
            nonlocal cursor
            cursor += 1
            send({"id": cursor, "method": method, "params": params})
            deadline = time.monotonic() + 15
            while True:
                response = json.loads(messages.get(timeout=max(0, deadline - time.monotonic())))
                if response.get("id") == cursor:
                    assert "error" not in response, response
                    return response["result"]

        try:
            init = call("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                       "clientInfo": {"name": "starter", "version": "0.1.0"}})
            assert init["protocolVersion"]
            send({"method": "notifications/initialized"})
            assert {"capabilities", "execute", "execute_batch"} <= {tool["name"] for tool in call("tools/list", {})["tools"]}
            state = call("tools/call", {"name": "capabilities", "arguments": {}})["structuredContent"]
            revision = state["revision"]
            names = {operation["id"] for operation in state["operations"]}
            for index, (op, params, doc, expected) in enumerate(scenario):
                assert op in names, op
                req = {"command_id": op, "parameters": params, "session_id": state["session_id"],
                       "expected_revision": revision, "document_id": doc, "request_id": f"r-{index}"}
                arg = {"name": "execute", "arguments": req}
                answer = call("tools/call", arg)
                assert not answer.get("isError"), answer
                receipt = answer["structuredContent"]
                revision = receipt["revision"]
                expect(receipt["result"], expected)
                assert call("tools/call", arg)["structuredContent"] == receipt
            denied = call("tools/call", {"name": "execute", "arguments": {**req, "session_id": "unknown"}})
            assert denied.get("isError") and denied["structuredContent"]["code"] == "forbidden"
        finally:
            peer.stdin.close()
            try:
                peer.wait(timeout=5)
            except subprocess.TimeoutExpired:
                peer.kill()
                peer.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", action="append", choices=SCENARIOS)
    parser.add_argument("--hybrid", action="store_true")
    parser.add_argument("--visual", action="store_true")
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.parent.mkdir(parents=True, exist_ok=True)
    status = []
    toolchain = subprocess.check_output(["rustc", "-vV"], text=True).strip()
    refs = read_json(ROOT / "examples/catalog.json")
    env = dict(os.environ)

    def check(name, command, run, artifacts=()):
        start = time.monotonic()
        state, message = "passed", ""
        try:
            run()
        except FileNotFoundError as error:
            state, message = "blocked", str(error)
        except (AssertionError, OSError, ValueError, queue.Empty, subprocess.SubprocessError) as error:
            state, message = "failed", f"{type(error).__name__}: {error}"
        status.append({"target": name, "command": command, "status": state, "message": message,
                       "duration_ms": round((time.monotonic()-start)*1000),
                       "artifacts": list(artifacts) if state == "passed" else []})
        evidence.write_text(json.dumps({"toolchain": toolchain, "checks": status}, indent=2) + "\n", encoding="utf-8")
        print(state, name, command, message, flush=True)
        return state == "passed"

    with tempfile.TemporaryDirectory(prefix="starter-validation-", dir=evidence.parent) as folder:
        work = Path(folder)
        target = Path(env.get("CARGO_TARGET_DIR", work / "target")).resolve()
        env["CARGO_TARGET_DIR"] = str(target)
        exe = target / "debug" / ("desktop.exe" if os.name == "nt" else "desktop")

        def cargo(project, *command):
            subprocess.run(["cargo", *command], cwd=project, env=env, check=True)

        if args.hybrid:
            project = work / "hybrid"
            if check("hybrid", "generate desktop-egui-web workbench",
                     lambda: generate("desktop-egui-web", "workbench", "hybrid-demo", project)):
                check("native", "cargo test --workspace", lambda: cargo(project, "test", "--workspace"))
                check("wasm32-unknown-unknown", "cargo check web and portable modules",
                      lambda: cargo(project, "check", "-p", "web", "-p", "starter-core", "-p", "starter-desktop",
                                    "-p", "starter-documents", "--target", "wasm32-unknown-unknown"))
        else:
            for name in args.reference or list(SCENARIOS):
                config = refs[name]
                project = work / name
                def compose(directory):
                    return generate(config["profile"], config["preset"], name, directory, config["modules"], reference=name)
                if not check(name, "generate", lambda: compose(project)):
                    continue
                def deterministic():
                    assert compose(work / f"{name}-copy") == read_json(project / "generation-report.json")
                check(name, "deterministic report and file hashes", deterministic)
                check(name, "cargo fmt --all -- --check", lambda: cargo(project, "fmt", "--all", "--", "--check"))
                check(name, "cargo xtask layers", lambda: cargo(project, "xtask", "layers"))
                if name == "data-explorer":
                    def invalid_layer():
                        manifest = project / "crates/starter-dataset/Cargo.toml"
                        original = manifest.read_text(encoding="utf-8")
                        try:
                            manifest.write_text(original + '\negui = "0.37"\n', encoding="utf-8")
                            result = subprocess.run(["cargo", "xtask", "layers"], cwd=project,
                                                    env=env, capture_output=True, text=True, check=False)
                            assert result.returncode != 0, result.stderr
                            assert "starter-demo-domain -> starter-dataset -> egui" in result.stderr, result.stderr
                        finally:
                            manifest.write_text(original, encoding="utf-8")

                    check(name, "cargo metadata rejects an injected transitive egui dependency", invalid_layer)
                check(name, "cargo test --workspace", lambda: cargo(project, "test", "--workspace"))
                check(name, "cargo clippy --workspace --all-targets -- -D warnings",
                      lambda: cargo(project, "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"))
                if check(name, "cargo build -p desktop", lambda: cargo(project, "build", "-p", "desktop")):
                    check(name, "CLI process", lambda: cli(exe, project, SCENARIOS[name]))
                    check(name, "MCP initialize, discover, execute and replay", lambda: mcp(exe, project, SCENARIOS[name]))
                if name == "document-workbench":
                    check(name, "cargo check -p desktop --features ui-inspection",
                          lambda: cargo(project, "check", "-p", "desktop", "--features", "ui-inspection"))
                if args.visual and name == "document-workbench":
                    env["STARTER_VISUAL_EVIDENCE"] = str(evidence.parent / "gallery")
                    check(name, "wgpu gallery and AccessKit interaction",
                          lambda: cargo(project, "test", "-p", "starter-desktop", "--features",
                                        "visual-evidence", "capture_gallery_interaction"), ["gallery"])
                if args.visual and name == "engineering-workbench":
                    env["STARTER_VIEWPORT_CAPTURE"] = str(evidence.parent / "viewport.png")
                    check(name, "wgpu offscreen viewport readback",
                          lambda: cargo(project, "test", "-p", "starter-viewport-wgpu", "--features",
                                        "gpu-evidence"), ["viewport.png"])
    return int(any(result["status"] != "passed" for result in status))


if __name__ == "__main__":
    sys.exit(main())
