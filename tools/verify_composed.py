#!/usr/bin/env python3
"""Generate and verify the modular references and record observed results."""
import argparse
import hashlib
import platform
import urllib.error
import urllib.request
import json
import os
from pathlib import Path
import queue
import socket
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


def mcp(exe, project, scenario, flags=("--mcp",)):
    with subprocess.Popen([str(exe), *flags], cwd=project, text=True, encoding="utf-8",
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
            stale = {**req, "request_id": "stale", "expected_revision": revision + 1}
            batch = call("tools/call", {"name": "execute_batch", "arguments": {"requests": [req, stale]}})
            results = batch["structuredContent"]
            assert results["atomic"] is False, batch
            assert "ok" in results["items"][0], batch
            assert results["items"][1]["error"]["code"] == "conflict", batch
        finally:
            peer.stdin.close()
            try:
                peer.wait(timeout=5)
            except subprocess.TimeoutExpired:
                peer.kill()
                peer.wait()


def http_service(exe, project, *, ssr=False):
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1",0))
        port = reservation.getsockname()[1]
    token = "local-integration-test-" + "4" * 32
    env = {**os.environ, "APP_BIND": f"127.0.0.1:{port}", "APP_OPERATIONS_TOKEN": token}
    base = f"http://127.0.0.1:{port}"
    transport = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with subprocess.Popen([str(exe)],cwd=project,env=env,stdout=subprocess.DEVNULL) as child:
        try:
            deadline = time.monotonic()+12
            while True:
                try:
                    with transport.open(base + ("/" if ssr else "/health"), timeout=1) as response:
                        if ssr:
                            assert b"Rendered on the Rust server" in response.read()
                        else:
                            assert json.load(response)["status"] == "ok"
                    break
                except urllib.error.URLError:
                    assert child.poll() is None and time.monotonic()<deadline, "service did not become reachable"
                    time.sleep(0.1)
            headers = {"Content-Type":"application/json","Authorization":"Bearer "+token}
            with transport.open(urllib.request.Request(base+"/operations",headers=headers),timeout=5) as response:
                state = json.load(response)
            request = {"command_id":"workspace.rename","session_id":state["session_id"],
                       "parameters":{"name":"Verified via HTTP"}, "expected_revision":state["revision"]}
            with transport.open(urllib.request.Request(base+"/invoke",json.dumps(request).encode(),headers),timeout=5) as response:
                assert json.load(response)["result"] == "Verified via HTTP"
        finally:
            child.terminate()
            try: child.wait(timeout=5)
            except subprocess.TimeoutExpired: child.kill(); child.wait()


def read_mcp_response(response):
    if response.headers.get_content_type() == "text/event-stream":
        for line in response:
            if line.startswith(b"data:") and line[5:].strip():
                return json.loads(line[5:].strip())
        raise AssertionError("MCP event stream ended without a message")
    content = response.read()
    if not content:
        raise AssertionError(f"MCP returned empty JSON body, status {response.status}")
    return json.loads(content)


def standalone_mcp(exe, project, scenario=None, *, client=None):
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        address = reservation.getsockname()
    env = {**os.environ, "MCP_BIND": f"{address[0]}:{address[1]}"}
    endpoint = f"http://{env['MCP_BIND']}/mcp"
    with subprocess.Popen([str(exe)], cwd=project, env=env, stdout=subprocess.DEVNULL) as server:
        try:
            deadline = time.monotonic() + 12
            while True:
                try:
                    with socket.create_connection(address, timeout=1):
                        break
                except OSError:
                    assert server.poll() is None and time.monotonic() < deadline, "MCP server did not bind"
                    time.sleep(0.1)
            if client:
                output = subprocess.check_output([str(client)], cwd=project, text=True, timeout=20,
                                                 env={**env, "MCP_URL": endpoint})
                assert "capabilities" in output and "execute_batch" in output, output
                return
            transport = urllib.request.build_opener(urllib.request.ProxyHandler({}))
            session = None

            def call(method, params=None, *, number=None):
                message = {"jsonrpc": "2.0", "method": method}
                if params is not None:
                    message["params"] = params
                if number is not None:
                    message["id"] = number
                headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
                if session:
                    headers["Mcp-Session-Id"] = session
                    headers["MCP-Protocol-Version"] = "2025-11-25"
                request = urllib.request.Request(endpoint, json.dumps(message).encode(), headers)
                with transport.open(request, timeout=8) as response:
                    return response.headers.get("Mcp-Session-Id"), (read_mcp_response(response) if number is not None else None)

            session, response = call("initialize", {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "starter-tests", "version": "0.1.0"},
            }, number=1)
            assert response["result"]["protocolVersion"] == "2025-11-25", response
            call("notifications/initialized")
            _, response = call("tools/list", {}, number=2)
            assert {"capabilities", "execute", "execute_batch"} <= {t["name"] for t in response["result"]["tools"]}
            _, response = call("tools/call", {"name": "capabilities", "arguments": {}}, number=3)
            status = response["result"]["structuredContent"]
            op, values, document, expected = scenario[0]
            _, response = call("tools/call", {"name": "execute", "arguments": {
                "command_id": op, "parameters": values, "document_id": document,
                "session_id": status["session_id"], "expected_revision": status["revision"],
            }}, number=4)
            expect(response["result"]["structuredContent"]["result"], expected)
        finally:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait()


def native_window(project, target, evidence, environment):
    if sys.platform.startswith("linux") and not (environment.get("DISPLAY") or environment.get("WAYLAND_DISPLAY")):
        raise FileNotFoundError("a display server is required for native window inspection")
    subprocess.run(["cargo","build","-p","desktop","--features","ui-inspection"],cwd=project,env=environment,check=True)
    exe = target / "debug" / ("desktop.exe" if os.name == "nt" else "desktop")
    with subprocess.Popen([str(exe),"--ui-inspect"],cwd=project,env=environment) as child:
        try:
            subprocess.run(["cargo","run","-p","starter-reference","--features","ui-inspection",
                            "--example","inspection_smoke","--",str(evidence)],
                           cwd=project,env=environment,check=True,timeout=200)
        finally:
            child.terminate()
            try: child.wait(timeout=5)
            except subprocess.TimeoutExpired: child.kill(); child.wait()


def compare_native_captures(project, evidence, environment):
    directory = evidence.parent
    reference = directory / "native-window.png"
    scene = directory / "native-scene.json"
    subprocess.run([sys.executable, "tools/init_scene.py", "--image", str(reference), "--out",
                    str(scene), "--pixels-per-point", "1", "--state", "second note created"],
                   cwd=project, env=environment, check=True)
    subprocess.run([sys.executable, "tools/visual_check.py", "--scene", str(scene),
                    "--candidate", str(directory / "native-repeat.png"),
                    "--out", str(directory / "native-forge")],
                   cwd=project, env=environment, check=True)


def identity():
    """Hash tracked and non-ignored source files, including worktree edits."""
    git = lambda *args: subprocess.check_output(["git", *args], cwd=ROOT)
    try:
        names = git("ls-files", "-z", "--cached", "--others", "--exclude-standard").split(b"\0")
        commit = git("rev-parse", "HEAD").decode().strip()
    except (FileNotFoundError, subprocess.CalledProcessError):
        return {"commit": None, "tree_sha256": None}
    digest = hashlib.sha256()
    for name in sorted(set(names) - {b""}):
        file = ROOT / os.fsdecode(name)
        if file.is_symlink():
            data = b"link:" + os.fsencode(os.readlink(file))
        elif file.is_file():
            data = hashlib.sha256(file.read_bytes()).digest()
        else:
            continue
        digest.update(len(name).to_bytes(4, "big") + name + len(data).to_bytes(4,"big") + data)
    return {"commit": commit, "tree_sha256": digest.hexdigest()}


def live_mcp(exe, project):
    token = "test-only-" + "7" * 32
    with tempfile.TemporaryDirectory(dir=project) as temp:
        env = {**os.environ, "STARTER_SESSION_DIR": temp, "STARTER_MCP_TOKEN": token}
        with subprocess.Popen([str(exe), "--mcp-live-headless"], cwd=project, env=env,
                              stdin=subprocess.PIPE, stderr=subprocess.PIPE) as process:
            try:
                deadline = time.monotonic() + 15
                while True:
                    found = subprocess.check_output([str(exe), "--sessions"], cwd=project, env=env)
                    instances = json.loads(found)
                    if instances: break
                    assert process.poll() is None, process.stderr.read().decode()
                    assert time.monotonic() < deadline, "live instance did not register"
                    time.sleep(0.05)
                [instance] = instances
                assert instance["application"] and instance["pid"] == process.pid, instance
                assert instance["address"].startswith("127.0.0.1:"), instance
                descriptor = Path(temp) / (instance["session_id"] + ".json")
                assert json.loads(descriptor.read_text()) == instance
                assert token not in descriptor.read_text()
                endpoint = f"http://{instance['address']}/mcp"
                client = urllib.request.build_opener(urllib.request.ProxyHandler({}))
                session_id = None
                def request(message, authorized=True, origin=None):
                    headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
                    if authorized: headers["Authorization"] = f"Bearer {token}"
                    if origin: headers["Origin"] = origin
                    if session_id: headers["Mcp-Session-Id"] = session_id
                    return client.open(urllib.request.Request(endpoint,
                        json.dumps({"jsonrpc": "2.0", **message}).encode(), headers), timeout=5)
                initialize = {"id": 1, "method": "initialize", "params": {
                    "protocolVersion": "2025-11-25", "capabilities": {},
                    "clientInfo": {"name": "starter-tests", "version": "0.1.0"}}}
                try:
                    request(initialize, False)
                except urllib.error.HTTPError as exc:
                    assert exc.code == 401, exc
                else:
                    raise AssertionError("unauthenticated live session")
                try:
                    request(initialize, origin="http://127.0.0.1")
                except urllib.error.HTTPError as exc:
                    assert exc.code == 403, exc
                else:
                    raise AssertionError("live MCP accepted a browser Origin")
                with request(initialize) as response:
                    session_id = response.headers.get("Mcp-Session-Id")
                    assert read_mcp_response(response)["result"]["protocolVersion"]
                with request({"method": "notifications/initialized"}):
                    pass
                with request({"id": 2, "method": "tools/call", "params": {"name": "capabilities", "arguments": {}}}) as response:
                    state = read_mcp_response(response)["result"]["structuredContent"]
                    assert state["session_id"] == instance["session_id"], state
                with request({"id": 3, "method": "tools/call", "params": {"name": "execute", "arguments": {
                    "command_id": "notes.edit", "session_id": state["session_id"], "document_id": "1",
                    "expected_revision": state["revision"], "parameters": {"text": "via live MCP"}}}}) as response:
                    assert read_mcp_response(response)["result"]["structuredContent"]["result"]["text"] == "via live MCP"
            finally:
                process.stdin.close()
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: process.kill(); process.wait()
        assert not descriptor.exists()
        descriptor.write_text(json.dumps(instance))
        assert json.loads(subprocess.check_output([str(exe), "--sessions"], cwd=project, env=env)) == []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", action="append", choices=SCENARIOS)
    parser.add_argument("--hybrid", action="store_true")
    parser.add_argument("--ordinary", action="store_true")
    parser.add_argument("--visual", action="store_true")
    parser.add_argument("--native-window", action="store_true")
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.parent.mkdir(parents=True, exist_ok=True)
    status = []
    toolchain = subprocess.check_output(["rustc", "-vV"], text=True).strip()
    refs = read_json(ROOT / "examples/catalog.json")
    source = identity()
    configuration = vars(args) | {"evidence": evidence.name}
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
        evidence.write_text(json.dumps({"toolchain": toolchain, "source": source, "platform": platform.platform(), "configuration": configuration, "checks": status}, indent=2) + "\n", encoding="utf-8")
        print(state, name, command, message, flush=True)
        return state == "passed"

    with tempfile.TemporaryDirectory(prefix="starter-validation-", dir=evidence.parent) as folder:
        work = Path(folder)
        target = Path(env.get("CARGO_TARGET_DIR", work / "target")).resolve()
        env["CARGO_TARGET_DIR"] = str(target)
        exe = target / "debug" / ("desktop.exe" if os.name == "nt" else "desktop")

        def cargo(project, *command):
            subprocess.run(["cargo", *command], cwd=project, env=env, check=True)

        if args.ordinary:
            workspace = [("workspace.rename", {"name": "Ordinary project"}, None, "Ordinary project")]
            profiles = read_json(ROOT / "profiles.json")
            configurations = [
                (profile, preset, ["application-mcp"] if (profile, preset) == ("cli", "application") else [])
                for preset in ("minimal", "application") for profile in profiles
            ]
            configurations.extend(("desktop-egui", preset, []) for preset in ("workbench", "engineering"))
            for profile, preset, modules in configurations:
                name = f"{profile}-{preset}"
                project = work / name
                if not check(name, "generate ordinary project", lambda: generate(profile, preset, name, project, modules)):
                    continue
                check(name, "generated documentation and skills", lambda: subprocess.run(
                    [sys.executable, str(ROOT / "tools/check_docs.py"), "--project", str(project)], check=True))
                descriptor = profiles[profile]
                if "wasm" in descriptor["targets"]:
                    wasm_args = ["-p", descriptor["wasm_package"]] if "wasm_package" in descriptor else []
                    check(name, "cargo check WASM", lambda: cargo(
                        project, "check", *wasm_args, "--target", "wasm32-unknown-unknown"))
                if "native" not in descriptor["targets"]:
                    continue
                check(name, "cargo test --workspace", lambda: cargo(project, "test", "--workspace"))
                if preset == "minimal" and profile != "cli":
                    continue
                if profile in {"library", "web-leptos-csr", "web-dioxus"}:
                    continue
                package = "desktop" if profile.startswith("desktop-egui") else name
                build = ["build", "-p", package, "--bins"]
                binary = target / "debug" / (package + (".exe" if os.name == "nt" else ""))
                if not check(name, "cargo " + " ".join(build), lambda: cargo(project, *build)):
                    continue
                if preset == "minimal":
                    check(name, "minimal greet", lambda: expect(subprocess.check_output(
                        [str(binary), "greet", "Rust"], cwd=project, text=True).strip(), "Hello, Rust!"))
                elif profile in {"service-axum", "web-leptos-fullstack"}:
                    check(name, "authenticated HTTP operation",
                          lambda: http_service(binary, project, ssr=profile == "web-leptos-fullstack"))
                elif profile == "mcp-stdio":
                    check(name, "standalone MCP operation", lambda: mcp(binary, project, workspace, flags=()))
                elif profile == "mcp-http":
                    check(name, "standalone Streamable HTTP operation",
                          lambda: standalone_mcp(binary, project, workspace))
                elif profile == "mcp-client":
                    companion = target / "debug" / ("operations.exe" if os.name == "nt" else "operations")
                    check(name, "MCP client's local companion operation", lambda: cli(companion, project, workspace))
                    server = target / "debug" / ("mcp-http-application.exe" if os.name == "nt" else "mcp-http-application")
                    check(name, "MCP client's external tool discovery", lambda: standalone_mcp(server, project, client=binary))
                else:
                    scenario = {
                        "workbench": SCENARIOS["document-workbench"],
                        "engineering": SCENARIOS["engineering-workbench"],
                    }.get(preset, workspace)
                    check(name, "ordinary CLI operation", lambda: cli(binary, project, scenario))
                    if "application-mcp" in modules:
                        check(name, "ordinary MCP operation", lambda: mcp(binary, project, scenario))
        elif args.hybrid:
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
                check(name, "generated documentation links", lambda: subprocess.run(
                    [sys.executable, str(ROOT / "tools/check_docs.py"), "--project", str(project)], check=True))
                check(name, "cargo fmt --all -- --check", lambda: cargo(project, "fmt", "--all", "--", "--check"))
                check(name, "cargo xtask layers", lambda: cargo(project, "xtask", "layers"))
                if name == "data-explorer":
                    check(name, "cargo xtask affected starter-core",
                          lambda: cargo(project, "xtask", "affected", "starter-core"))
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
                    check(name, "MCP initialize, discover, execute, replay and batch", lambda: mcp(exe, project, SCENARIOS[name]))
                if name == "document-workbench":
                    check(name, "live loopback MCP discovery, authorization and disconnect", lambda: live_mcp(exe, project))
                    check(name, "cargo check -p desktop --features ui-inspection",
                          lambda: cargo(project, "check", "-p", "desktop", "--features", "ui-inspection"))
                    check(name, "official inspection plugin and command palette tests",
                          lambda: cargo(project,"test","-p","starter-reference","--features","ui-inspection"))
                if args.native_window and name == "document-workbench":
                    if check(name, "native inspection discovers and clicks a control, then captures the window",
                             lambda: native_window(project, target, evidence.parent / "native-window.png", env),
                             ["native-before.txt", "native-after.txt", "native-before.png",
                              "native-window.png", "native-repeat.png"]):
                        check(name, "Visual Forge compares captures of the post-interaction state",
                              lambda: compare_native_captures(project, evidence, env),
                              ["native-scene.json", "native-forge"])
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
