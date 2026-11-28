#!/usr/bin/env python3
"""Discover capabilities and generate an autonomous Rust application."""

import argparse
import importlib.util
import platform
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

from generator import compose, integration

ROOT = compose.ROOT
PROFILES = compose.read_json(ROOT / "profiles.json")
PRESETS = compose.read_json(ROOT / "presets/catalog.json")
PRODUCTS = compose.read_json(ROOT / "presets/products.json")
CAPABILITIES = compose.read_json(ROOT / "presets/capabilities.json")
WASM = "wasm32-unknown-unknown"


def plan(request):
    for key in ("profile", "product", "preset", "name", "destination"):
        if key in request and not isinstance(request[key], str):
            raise compose.CompositionError(f"{key} must be a string")
    for key in ("capabilities", "modules", "features", "targets"):
        if key in request and (not isinstance(request[key], list)
                               or not all(isinstance(item, str) for item in request[key])):
            raise compose.CompositionError(f"{key} must be a list of strings")
    product = PRODUCTS.get(request.get("product"))
    if "product" in request and product is None:
        raise compose.CompositionError(f"unknown product: {request['product']}")
    profile = request.get("profile") or (product["profile"] if product else None)
    if not profile:
        raise compose.CompositionError("choose a profile or a product")
    if product and (request.get("preset") or profile != product["profile"]):
        raise compose.CompositionError("the product already selects its profile and preset")
    preset = product["preset"] if product else request.get("preset") or "minimal"
    capabilities = list(dict.fromkeys([*(product["capabilities"] if product else []), *request.get("capabilities", [])]))
    requested = list(request.get("modules", []))
    for name in capabilities:
        if name not in CAPABILITIES:
            raise compose.CompositionError(f"unknown capability: {name}")
        capability = CAPABILITIES[name]
        if "profiles" in capability and profile not in capability["profiles"]:
            raise compose.CompositionError(f"{name} is incompatible with {profile}")
        requested.extend(capability["modules"])
    views = set(capabilities).intersection({"documents", "inspection-reports", "data-visualization", "engineering-views"})
    if preset in {"workbench", "engineering"}:
        views.add("documents" if preset == "workbench" else "engineering-views")
    spec = {"profile": profile, "preset": preset, "name": request.get("name") or "my-app",
            "requested": sorted(set(requested)), "targets": request.get("targets", []),
            "features": request.get("features", [])}
    descriptor, targets, modules, ordered, features = compose.resolve_configuration(**spec)
    expected_workspace = {"inspection-reports": "inspection-demo", "data-visualization": "data-demo", "engineering-views": "engineering-demo"}
    if profile == "desktop-egui":
        expected_workspace["documents"] = "document-demo"
    combined_views = {"documents", "data-visualization", "engineering-views"} if "lab-workbench" in ordered else set()
    for capability, provider in expected_workspace.items():
        if capability in views - combined_views and provider not in ordered:
            raise compose.CompositionError(f"{capability} requires the {provider} workspace")
    providers = frozenset(ordered).intersection({"desktop-product", "document-demo", "inspection-demo", "data-demo", "engineering-demo", "lab-workbench"})
    external = {}
    crates = set()
    manifests = {
        str(manifest.relative_to(ROOT / "templates" / profile)): tomllib.loads(manifest.read_text())
        for manifest in (ROOT / "templates" / profile).rglob("Cargo.toml")
    }
    for name in ordered:
        for path in modules[name]["cargo_dependencies"]:
            manifests[path] = tomllib.loads((ROOT / "modules" / name / "files" / path).read_text())
    for name in ordered:
        for path, fragment in modules[name].get("manifests", {}).items():
            compose.merge(manifests.setdefault(path, {}), fragment)
    for manifest in manifests.values():
        if package := manifest.get("package"):
            crates.add(package["name"].replace("{{project-name}}", spec["name"]))
        for section in [manifest, *manifest.get("target", {}).values(), manifest.get("workspace", {})]:
            for table in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, value in section.get(table, {}).items():
                    if isinstance(value, str):
                        external.setdefault(name, set()).add(value)
                    elif "path" not in value and not value.get("workspace"):
                        external.setdefault(value.get("package", name), set()).add(
                            value.get("version") or value.get("rev") or "unspecified")
    selected = set(ordered)
    included = []
    for name, capability in CAPABILITIES.items():
        if not set(capability["modules"]) <= selected:
            continue
        if profile not in capability.get("profiles", PROFILES):
            continue
        if name in expected_workspace and name not in combined_views and expected_workspace[name] not in selected:
            continue
        if name == "background-work" and not providers:
            continue
        included.append(name)
    return {
        "request": request, "composition": spec, "targets": targets, "profile_targets": descriptor["targets"],
        "capabilities": included, "requested_capabilities": capabilities, "providers": sorted(providers),
        "modules": {name: modules[name]["version"] for name in ordered},
        "crates": sorted(crates), "declared_dependency_constraints": {
            name: sorted(versions) for name, versions in sorted(external.items())
        },
        "features": sorted(set(spec["features"])), "available_features": sorted(features), "entrypoints": integration.entrypoints(profile, ordered, modules),
        "requirements": ["Python 3.11+", "tomli-w", "Rust", "cargo-generate", "rustfmt"],
        "limits": [limit for name in included for limit in CAPABILITIES[name].get("limitations", [])],
    }


def command_output(command):
    if not shutil.which(command[0]):
        return None
    try:
        env = {**os.environ, "TELEMETRY": "false"} if command[0] == "dx" else None
        result = subprocess.run(command, capture_output=True, text=True, check=False, timeout=10, env=env)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return result.stdout.strip() if result.returncode == 0 else None


def command_available(command):
    return command_output(command) is not None


def tool_version(text):
    match = re.search(r"\b(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.-]+))?", text or "")
    return (tuple(map(int, match.groups()[:3])), match.group(4)) if match else None


def doctor(preview=None, verify=False):
    targets = preview["targets"] if preview else []
    profile = preview["composition"]["profile"] if preview else None
    required = [
        ("cargo", ["cargo", "--version"], "Install Rust with rustup (https://rustup.rs)."),
        ("rustc", ["rustc", "--version"], "Install the stable Rust toolchain with rustup."),
        ("cargo-generate", ["cargo", "generate", "--version"], "cargo install cargo-generate --version 0.26.0 --locked"),
        ("rustfmt", ["cargo", "fmt", "--version"], "rustup component add rustfmt"),
    ]
    # Cargo Generate renders templates even when the destination will be built later.
    # Edition 2024 requires 1.85. The official egui 0.37 manifest raises that to 1.95.
    rust_minimum = (1, 95, 0) if profile in integration.DESKTOP else (1, 85, 0)
    minimums = {"cargo": rust_minimum, "rustc": rust_minimum, "cargo-generate": (0, 26, 0)}
    checks = []
    for key, command, hint in required:
        version_text = command_output(command)
        observed = tool_version(version_text)
        minimum = minimums.get(key)
        ok = version_text is not None and (not minimum or (observed is not None and observed[0] >= minimum))
        checks.append({"id": key, "ok": ok, "required": True, "hint": hint,
                       "version": version_text, "minimum": ".".join(map(str, minimum)) if minimum else None})
    checks.append({"id": "python", "ok": sys.version_info >= (3, 11), "required": True,
                   "version": platform.python_version(), "hint": "Use Python 3.11 or newer."})
    checks.append({"id": "tomli-w", "ok": importlib.util.find_spec("tomli_w") is not None,
                   "required": True, "hint": "uv run --with-requirements generator/requirements.txt python starter.py ..."})
    checks.append({"id": "uv", "ok": shutil.which("uv") is not None,
                   "required": False, "hint": "Install uv for isolated generator and Visual Forge tooling."})
    checks.append({"id": "clippy", "ok": command_available(["cargo", "clippy", "--version"]),
                   "required": False, "hint": "rustup component add clippy"})
    if "wasm" in targets:
        rustc = shutil.which("rustc")
        target_lib = subprocess.run([rustc, "--print", "target-libdir", "--target", WASM], capture_output=True,
                                     encoding="utf-8", check=False) if rustc else None
        has_std = target_lib is not None and target_lib.returncode == 0 and any(
            Path(target_lib.stdout.strip()).glob("libstd-*.rlib"))
        checks.append({"id": WASM, "ok": has_std, "required": verify,
                       "hint": f"rustup target add {WASM}"})
        web_tool = "dx" if profile == "web-dioxus" else "trunk"
        installed = command_output([web_tool, "--version"])
        if web_tool == "dx":
            manifest = tomllib.loads((ROOT / "templates/web-dioxus/Cargo.toml").read_text(encoding="utf-8"))
            runtime_version = manifest["dependencies"]["dioxus"]["version"].removeprefix("=")
            ok = tool_version(installed) == tool_version(runtime_version)
            hint = f"cargo install dioxus-cli --version {runtime_version} --locked"
        else:
            ok = installed is not None
            hint = "cargo install trunk --version 0.22.2 --locked"
        checks.append({"id": web_tool, "ok": ok, "required": False, "version": installed, "hint": hint})
    if profile and profile.startswith("desktop-egui") and "native" in targets and sys.platform.startswith("linux"):
        checks.append({"id": "linux-window-libraries", "ok": command_available(["pkg-config", "--exists", "xkbcommon", "wayland-client"]),
                       "required": False, "hint": "On Debian or Ubuntu install libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev libx11-dev for native windows."})
    return checks


def verify_project(project, report):
    commands = [[sys.executable, str(ROOT / "tools/check_docs.py"), "--project", str(project)]]
    feature_flags = ["--features", ",".join(report["features"])] if report["features"] else []
    if "native" in report["targets"]:
        commands.append(["cargo", "test", "--workspace", *feature_flags])
    if "wasm" in report["targets"]:
        package = ["-p", report["wasm_package"]] if report["wasm_package"] else []
        # A native desktop feature belongs to the desktop invocation in a hybrid.
        wasm_features = feature_flags if "native" not in report["targets"] else []
        commands.append(["cargo", "check", *package, "--target", WASM, *wasm_features])
    for command in commands:
        subprocess.run(command, cwd=project, check=True, stdout=sys.stderr)


def choose(label, options, default):
    print(label, file=sys.stderr)
    for i, (name, title) in enumerate(options.items(), start=1):
        print(f"  {i}. {name} — {title}", file=sys.stderr)
    choice = input(f"Select [{default}]: ").strip()
    names = list(options)
    if choice.isdigit() and 1 <= int(choice) <= len(names):
        return names[int(choice) - 1]
    if choice and choice not in options:
        raise compose.CompositionError(f"invalid selection: {choice}")
    return choice or default


def interactive_request():
    options = {name: item["description"] for name, item in PRODUCTS.items()}
    options.update({name: item["desc"] for name, item in PROFILES.items()})
    selection = choose("Application type", options, "desktop-application")
    product = PRODUCTS.get(selection)
    request = {"product": selection} if product else {
        "profile": selection, "preset": choose("Preset", {"minimal": "Base template", "application": "Typed operations and development tools"}, "application")
    }
    request["name"] = input("Project name: ").strip()
    if not request["name"]:
        raise compose.CompositionError("project name is required")
    request["destination"] = input(f"Destination [./{request['name']}]: ").strip() or request["name"]
    preview = plan(request)
    if len(preview["profile_targets"]) > 1:
        target = choose("Verify which targets?", {"all": "Native and WASM", "native": "Native", "wasm": "WASM"}, "all")
        if target != "all":
            request["targets"] = [target]
    supported = {}
    for name, spec in CAPABILITIES.items():
        try:
            plan({**request, "capabilities": [name]})
        except ValueError:
            continue
        if name not in preview["capabilities"]:
            supported[name] = spec["description"]
    for index, (name, title) in enumerate(supported.items(), start=1):
        print(f"  {index}. {name} — {title}", file=sys.stderr)
    choices = input("Additional capabilities (comma-separated numbers or IDs): ").strip()
    ids = list(supported)
    request["capabilities"] = [
        ids[int(item) - 1] if item.isdigit() and 1 <= int(item) <= len(ids) else item
        for token in choices.split(",") if (item := token.strip())
    ]
    return request


def emit(data, as_json):
    if as_json:
        print(json.dumps(data, indent=2))
    elif "composition" in data:
        spec = data["composition"]
        print(f"{spec['name']} · {spec['profile']} · {spec['preset']}")
        for heading, key in (("Capabilities", "capabilities"), ("Targets", "targets"), ("Entrypoints", "entrypoints"),
                             ("Workspace crates", "crates"), ("Requirements", "requirements")):
            print(f"{heading}: {', '.join(data[key]) or '—'}")
        for limit in data["limits"]:
            print(f"- {limit}")
        print("Use plan --json for features and dependency constraints.")
    else:
        for name, value in data.items():
            print(f"{name}: {json.dumps(value, ensure_ascii=False) if isinstance(value, (dict, list, bool)) else value}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    lists = {"list-profiles": PROFILES, "list-presets": {
        **{name: {"modules": modules} for name, modules in PRESETS.items()}, **PRODUCTS,
    }, "list-capabilities": CAPABILITIES}
    for name in [*lists, "describe"]:
        cmd = sub.add_parser(name)
        if name == "describe":
            cmd.add_argument("id")
        cmd.add_argument("--json", action="store_true")
    for name in ("plan", "create", "doctor"):
        cmd = sub.add_parser(name)
        cmd.add_argument("--json", action="store_true")
        cmd.add_argument("--config", type=Path, help="reuse the request from a previous plan")
        choice = cmd.add_mutually_exclusive_group()
        choice.add_argument("--profile", choices=PROFILES)
        choice.add_argument("--product", choices=PRODUCTS)
        cmd.add_argument("--preset", choices=PRESETS)
        cmd.add_argument("--capability", action="append", dest="capabilities")
        cmd.add_argument("--module", action="append", dest="modules", help="advanced module override")
        cmd.add_argument("--feature", action="append", dest="features")
        cmd.add_argument("--target", action="append", choices=["native", "wasm"], dest="targets")
        cmd.add_argument("--name")
        cmd.add_argument("--destination")
        cmd.add_argument("--verify", action="store_true", help="run checks for the selected targets")
    args = parser.parse_args()
    try:
        if args.action in lists:
            emit(lists[args.action], args.json)
            return 0
        if args.action == "describe":
            found = {name.removeprefix("list-"): items[args.id] for name, items in lists.items() if args.id in items}
            if not found:
                raise compose.CompositionError(f"unknown identifier: {args.id}")
            emit(found, args.json)
            return 0
        keys = ("profile", "product", "preset", "capabilities", "modules", "features", "targets", "name", "destination")
        request = {key: getattr(args, key) for key in keys if getattr(args, key) is not None}
        interactive = args.action == "create" and not request and not args.config and sys.stdin.isatty() and not args.json
        if args.config:
            if request:
                parser.error("--config cannot be combined with selection flags")
            config = compose.read_json(args.config)
            request = config.get("request", config)
            if set(request) - set(keys):
                raise compose.CompositionError("unsupported field in project configuration")
        if interactive:
            request = interactive_request()
        preview = plan(request) if request or args.action != "doctor" else None
        if args.action == "plan":
            emit(preview, args.json)
            return 0
        checks = doctor(preview, args.verify or interactive)
        if args.action == "doctor":
            emit({"checks": checks}, args.json)
            return int(any(item["required"] and not item["ok"] for item in checks))
        if missing := [f"{item['id']} — {item['hint']}" for item in checks if item["required"] and not item["ok"]]:
            raise compose.CompositionError("missing requirements: " + "; ".join(missing))
        if not request.get("name"):
            raise compose.CompositionError("create requires a project name")
        destination = Path(request.get("destination") or request["name"]).expanduser()
        if interactive:
            emit(preview, False)
            if input("Generate and verify this project? [y/N]: ").strip().lower() not in {"y", "yes", "s", "si", "sí"}:
                return 0
        report = compose.generate(**preview["composition"], destination=destination,
                                  capabilities=preview["capabilities"], product=request.get("product"),
                                  verify=verify_project if args.verify or interactive else None)
        emit({"destination": str(destination.resolve()), "verified": args.verify or interactive,
              "profile": report["profile"], "entrypoints": report["entrypoints"],
              "start": "docs/GETTING_STARTED.md"}, args.json)
        return 0
    except (ValueError, KeyError, TypeError, OSError, EOFError, subprocess.SubprocessError) as error:
        if args.json:
            emit({"error": str(error)}, True)
        else:
            print(f"starter: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
