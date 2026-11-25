"""Compose autonomous Cargo Generate profiles using local, versioned modules."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

if __package__:
    from . import integration
else:
    import integration

ROOT = Path(__file__).resolve().parents[1]
NAME = re.compile(r"[a-z][a-z0-9-]*\Z")
REQUIREMENT = re.compile(r"([a-z][a-z0-9-]*)(?:@(\d+\.\d+\.\d+))?\Z")
RESERVED = {"con", "prn", "aux", "nul", *(f"{kind}{i}" for kind in ("com", "lpt") for i in range(1, 10))}


class CompositionError(ValueError):
    pass


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def checked_path(filename):
    path = PurePosixPath(filename)
    if not path.parts or path.as_posix() != filename or any(
        part in (".", "..") or part.casefold() == ".git"
        or part.endswith(".") or part.split(".")[0].casefold() in RESERVED
        or not re.fullmatch(r"[a-zA-Z0-9_.-]+", part)
        for part in path.parts
    ):
        raise CompositionError(f"unsafe path: {filename}")
    return path


def catalog(root=ROOT):
    modules = {}
    for file in sorted((root / "modules").glob("*/module.json")):
        module = read_json(file)
        name = module["id"]
        if name != file.parent.name or REQUIREMENT.fullmatch(f"{name}@{module['version']}") is None:
            raise CompositionError(f"invalid module identifier in {file}")
        for filename in [*module["files"], *module.get("manifests", {})]:
            checked_path(filename)
        payload = file.parent / "files"
        actual = set()
        for item in payload.rglob("*"):
            relative = item.relative_to(payload)
            if item.is_symlink() or payload.is_symlink():
                raise CompositionError(f"symbolic links in module {name}")
            if item.is_file() and "__pycache__" not in relative.parts:
                actual.add(relative.as_posix())
        if actual != set(module["files"]):
            raise CompositionError(f"payload differs from descriptor for {name}")
        modules[name] = module
    return modules


def resolve(modules, requested, profile, targets):
    active, ordered = [], []

    def visit(requirement):
        match = REQUIREMENT.fullmatch(requirement)
        if not match:
            raise CompositionError(f"invalid module requirement: {requirement}")
        name, version = match.groups()
        if name not in modules:
            raise CompositionError(f"unknown module: {name}")
        module = modules[name]
        if version is not None and module["version"] != version:
            raise CompositionError(f"version mismatch for {requirement}")
        if name in active:
            raise CompositionError(f"dependency cycle: {' -> '.join([*active, name])}")
        if name in ordered:
            return
        if module["profiles"] and profile not in module["profiles"]:
            raise CompositionError(f"{name} is incompatible with profile {profile}")
        if not targets <= set(module["targets"]):
            raise CompositionError(f"{name} does not support {', '.join(sorted(targets-set(module['targets'])))}")
        active.append(name)
        for dependency in sorted(module["dependencies"]):
            visit(dependency)
        active.pop()
        ordered.append(name)

    for requirement in sorted(set(requested)):
        visit(requirement)
    for name in ordered:
        conflict = set(modules[name]["conflicts"]).intersection(ordered)
        if conflict:
            raise CompositionError(f"{name} conflicts with {', '.join(sorted(conflict))}")
    return ordered


def merge(target, incoming, path=()):
    for name, value in incoming.items():
        key = (*path, name)
        if name not in target:
            target[name] = value
        elif isinstance(target[name], dict) and isinstance(value, dict):
            merge(target[name], value, key)
        elif key == ("workspace", "members"):
            target[name] = list(dict.fromkeys([*target[name], *value]))
        elif target[name] != value:
            raise CompositionError(f"manifest conflict at {'.'.join(key)}")


def check_files(project, files, replace):
    known = {p.relative_to(project).as_posix().casefold(): p for p in project.rglob("*")}
    claimed, directories = {}, {}
    for filename in sorted(files):
        name = filename.casefold()
        if name in claimed or name in directories:
            raise CompositionError(f"file collision: {filename}")
        path = checked_path(filename)
        existing = known.get(name)
        if existing:
            if filename not in replace or existing.is_symlink() or not existing.is_file() or existing.relative_to(project).as_posix() != filename:
                raise CompositionError(f"file already exists: {filename}")
        elif filename in replace:
            raise CompositionError(f"replacement file missing: {filename}")
        claimed[name] = filename
        for parent in path.parents:
            if not parent.parts:
                continue
            spelling = parent.as_posix()
            folded = spelling.casefold()
            existing = known.get(folded)
            if (folded in claimed or directories.setdefault(folded, spelling) != spelling
                or (existing and (existing.is_symlink() or not existing.is_dir()
                    or existing.relative_to(project).as_posix() != spelling))):
                raise CompositionError(f"directory collision: {spelling}")


def verify_autonomy(project):
    def walk(data, manifest):
        if isinstance(data, dict):
            if "git" in data and not re.fullmatch(r"[0-9a-f]{40}", data.get("rev", "")):
                raise CompositionError(f"unpinned Git dependency in {manifest}")
            if isinstance(data.get("path"), str):
                path = Path(data["path"])
                if path.is_absolute() or not (manifest.parent / path).resolve().is_relative_to(project):
                    raise CompositionError(f"external Cargo path in {manifest}")
            for value in data.values():
                walk(value, manifest)
        if isinstance(data, list):
            for value in data:
                walk(value, manifest)

    for path in project.rglob("*"):
        if path.is_symlink() and not path.resolve().is_relative_to(project):
            raise CompositionError(f"escaping symlink {path.relative_to(project)}")
        if path.name == "Cargo.toml" and path.is_file():
            walk(tomllib.loads(path.read_text(encoding="utf-8")), path)


def composition_guide(profile, preset, interfaces, modules):
    entries = ", ".join(interfaces) or "base profile"
    lines = [
        "# Generated composition", "", f"Profile: `{profile}`. Preset: `{preset}`.",
        "", f"Entry points: {entries}.", "",
    ]
    guides = integration.guides(profile, interfaces)
    if guides:
        lines.extend(["## Installed entrypoints", "", *[f"- {item}" for item in guides], ""])
    lines.extend(
        f"- [{key}]({module['documentation'].removeprefix('docs/')}) — {module['description']}"
        for key, module in modules.items()
    )
    return "\n".join(lines) + "\n"


def generate(profile, preset, name, destination, requested=(), targets=(), features=(), reference=None):
    import tomli_w

    profiles = read_json(ROOT / "profiles.json")
    presets = read_json(ROOT / "presets/catalog.json")
    if profile not in profiles or preset not in presets:
        raise CompositionError(f"invalid profile or preset: {profile}/{preset}")
    if not NAME.fullmatch(name) or name in RESERVED | {"build", "deps", "examples", "incremental", "test", "xtask"}:
        raise CompositionError(f"invalid project name: {name}")
    valid_targets = set(profiles[profile]["targets"])
    selected_targets = set(targets) or valid_targets
    if not selected_targets <= valid_targets:
        raise CompositionError(f"incompatible targets: {sorted(selected_targets)}")
    modules = catalog()
    requested = [*presets[preset], *requested]
    ordered = resolve(modules, requested, profile, valid_targets)
    while needed := integration.plan(profile, preset, ordered, modules):
        requested.extend(sorted(needed))
        ordered = resolve(modules, requested, profile, valid_targets)
    declared_features = {feature for module in ordered for feature in modules[module]["features"]}
    if "reference-runtime" in ordered:
        declared_features.add("desktop/ui-inspection")
    if set(features) - declared_features:
        raise CompositionError(f"unsupported features: {sorted(set(features)-declared_features)}")

    files, replacements, manifests = {}, set(), {}
    for name_id in ordered:
        module = modules[name_id]
        for filename in module["files"]:
            if filename in files:
                raise CompositionError(f"file collision: {files[filename]} and {name_id} provide {filename}")
            files[filename] = name_id
        replacements.update(module.get("replace", {}).get(profile, []))
        for filename, data in module.get("manifests", {}).items():
            merge(manifests.setdefault(filename, {}), data)
    if not replacements <= files.keys():
        raise CompositionError("undeclared replacement files")

    destination = Path(destination).absolute()
    if destination.exists() or destination.is_symlink() or not destination.parent.is_dir():
        raise CompositionError(f"destination must be new and its parent must exist: {destination}")
    with tempfile.TemporaryDirectory(prefix=".starter-", dir=destination.parent) as tmp:
        subprocess.run(["cargo", "generate", "--path", str(ROOT / "templates" / profile),
                        "--name", name, "--destination", tmp, "--vcs", "none"], check=True)
        project = (Path(tmp) / name).resolve()
        check_files(project, files, replacements)
        for relative, module in files.items():
            target = project / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / "modules" / module / "files" / relative, target)
        fragments, interfaces = integration.install(project, profile, ordered, modules, name=name, merge=merge)
        for filename, fragment in fragments.items():
            merge(manifests.setdefault(filename, {}), fragment)
        for filename, fragment in manifests.items():
            path = project / filename
            if path.is_symlink() or not path.resolve().is_relative_to(project):
                raise CompositionError(f"unsafe manifest path: {filename}")
            manifest = tomllib.loads(path.read_text(encoding="utf-8"))
            merge(manifest, fragment)
            path.write_text(tomli_w.dumps(manifest), encoding="utf-8", newline="\n")
        if ordered:
            module_doc = project / "docs/COMPOSITION.md"
            module_doc.write_text(composition_guide(profile, preset, interfaces, {
                item: modules[item] for item in ordered
            }), encoding="utf-8")
            for doc in ("AGENTS.md", "README.md"):
                with (project / doc).open("a", encoding="utf-8") as stream:
                    stream.write("\nSee [the generated composition](docs/COMPOSITION.md) for installed capabilities.\n")
        subprocess.run([sys.executable, "scripts/check_agents.py"], cwd=project, check=True)
        verify_autonomy(project)
        metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=project, encoding="utf-8",
        ))
        installed_features = {
            f"{package['name']}/{feature}" for package in metadata["packages"]
            for feature in package["features"]
        }
        if missing := declared_features - installed_features:
            raise CompositionError(f"missing Cargo features after composition: {sorted(missing)}")
        if interfaces:
            subprocess.run(["cargo", "fmt", "--all"], cwd=project, check=True)
        report = {
            "schema_version": 2, "name": name, "profile": profile, "preset": preset,
            "targets": sorted(selected_targets), "profile_targets": sorted(valid_targets),
            "wasm_package": profiles[profile].get("wasm_package"), "features": sorted(set(features)),
            "reference": reference, "entrypoints": interfaces, "modules": {item: modules[item]["version"] for item in ordered},
            "required_checks": sorted({check for item in ordered for check in modules[item]["checks"]}),
            "sha256": {p.relative_to(project).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                       for p in sorted(project.rglob("*")) if p.is_file() and not p.is_symlink()},
        }
        (project / "generation-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        if destination.exists() or destination.is_symlink():
            raise CompositionError(f"destination already exists: {destination}")
        project.rename(destination)
        return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    profile = parser.add_mutually_exclusive_group(required=True)
    profile.add_argument("--profile")
    profile.add_argument("--reference")
    parser.add_argument("--preset")
    parser.add_argument("--module", action="append", default=[])
    parser.add_argument("--target", action="append", choices=["native", "wasm"], default=[], help="record verification target; keeps the profile entrypoints")
    parser.add_argument("--feature", action="append", default=[], help="record an optional package/feature for reproducibility; pass it to Cargo when building")
    parser.add_argument("--name", required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    try:
        selected = {"profile": args.profile, "preset": args.preset or "minimal", "modules": []}
        if args.reference:
            if args.preset:
                parser.error("a reference already selects a preset")
            selected = read_json(ROOT / "examples/catalog.json")[args.reference]
        generate(selected["profile"], selected["preset"], args.name, args.destination,
                 [*selected["modules"], *args.module], args.target, args.feature, args.reference)
    except (ImportError, ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"generation failed: {error}\n")
    print(f"Generated {args.destination}")


if __name__ == "__main__":
    main()
