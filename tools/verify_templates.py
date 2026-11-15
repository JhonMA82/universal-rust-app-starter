#!/usr/bin/env python3
"""Verify starter profiles offline; optionally generate with cargo-generate and compile.

Only uses Python stdlib for the offline checks. --generate/--cargo-check require
Rust tooling installed separately and never upload or publish anything.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILES = json.loads((ROOT / "profiles.json").read_text(encoding="utf-8"))
RENDER = re.compile(r"\{\{\s*project-name\s*\}\}")
SKILL_FIELDS = ("name", "description")


def read_toml(path: Path):
    return tomllib.loads(RENDER.sub("smoke-project", path.read_text(encoding="utf-8")))


def run(cmd: list[str], cwd: Path):
    print("  $", " ".join(map(str, cmd)), "in", cwd)
    subprocess.run(cmd, cwd=cwd, check=True)


def check_templates() -> list[str]:
    errors: list[str] = []
    actual = {x.name for x in (ROOT / "templates").iterdir() if x.is_dir()}
    if actual != set(PROFILES):
        errors.append(f"Registry/profile mismatch: expected {sorted(PROFILES)}, got {sorted(actual)}")

    for name, meta in PROFILES.items():
        base = ROOT / "templates" / name
        for item in ["Cargo.toml", "cargo-generate.toml", "README.md", "AGENTS.md", "CLAUDE.md",
                     "GEMINI.md", "docs/ARCHITECTURE.md", "docs/SKILLS.md",
                     ".github/workflows/ci.yml", ".github/workflows/security.yml",
                     "scripts/check_agents.py", "LICENSE-MIT", "LICENSE-APACHE"]:
            if not (base / item).is_file():
                errors.append(f"{name}: missing {item}")
        for man in base.rglob("Cargo.toml"):
            try:
                tree = read_toml(man)
                if "package" not in tree and "workspace" not in tree:
                    errors.append(f"{name}: missing package/workspace in {man.relative_to(base)}")
                if "package" in tree:
                    pkg=tree["package"]
                    inherited=pkg.get("edition", "2024")
                    if inherited not in ("2024", {"workspace": True}):
                        errors.append(f"{name}: edition not 2024/inherited in {man.relative_to(base)}")
            except Exception as exc:
                errors.append(f"{name}: TOML {man.relative_to(base)}: {exc}")
        for t in base.rglob("*.toml"):
            try:
                read_toml(t)
            except Exception as exc:
                errors.append(f"{name}: TOML {t.relative_to(base)}: {exc}")
        src = base / ".agents" / "skills"
        mirror = base / ".claude" / "skills"
        skills = sorted(x.parent.name for x in src.glob("*/SKILL.md"))
        mirrored = sorted(x.parent.name for x in mirror.glob("*/SKILL.md")) if mirror.exists() else []
        expected = sorted(["idiomatic-rust", "ownership-not-clone", "rust-errors", "rust-api-design",
                           "rust-testing", "rust-code-review", "rust-supply-chain", "rust-docs"] + meta["skills"])
        if skills != expected:
            errors.append(f"{name}: skills do not match profile expectations")
        if not mirror.is_symlink():
            errors.append(f"{name}: .claude/skills must be a symlink")
        else:
            target = Path(os.readlink(mirror))
            if target != Path("..") / ".agents" / "skills":
                errors.append(f"{name}: .claude/skills points to {target}")
            if mirror.resolve() != src.resolve():
                errors.append(f"{name}: .claude/skills does not resolve to .agents/skills")
        if skills != mirrored:
            errors.append(f"{name}: Claude skills are incomplete")
        for skill in skills:
            a = src / skill / "SKILL.md"
            text = a.read_text(encoding="utf-8")
            front = re.match(r"^---\n(.*?)\n---\n", text, re.S)
            if not front or not all(re.search(fr"^{field}:\s*\S+", front.group(1), re.M) for field in SKILL_FIELDS):
                errors.append(f"{name}: invalid frontmatter {skill}")
        for needed in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"]:
            if (base / needed).is_file():
                doc = (base / needed).read_text()
                if "{%" in doc:
                    errors.append(f"{name}: Liquid control tags in {needed}")
        if "@AGENTS.md" not in (base / "CLAUDE.md").read_text():
            errors.append(f"{name}: CLAUDE.md does not import contract")
        if "@AGENTS.md" not in (base / "GEMINI.md").read_text():
            errors.append(f"{name}: GEMINI.md does not import contract")
        # Run the distributed validator itself, not a reimplementation.
        try:
            subprocess.run([sys.executable, str(base / "scripts" / "check_agents.py")],
                           check=True, cwd=base, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        except (subprocess.CalledProcessError, OSError) as exc:
            errors.append(f"{name}: distributed agent readiness failed: {exc}")
        for cfg in [base / ".github/workflows/ci.yml", base / ".github/workflows/security.yml"]:
            text = cfg.read_text()
            if "permissions:\n  contents: read" not in text:
                errors.append(f"{name}: no read-only default workflow permissions")
            if "pull_request" not in text:
                errors.append(f"{name}: workflow missing pull_request trigger")
            if "@v3" in text:
                errors.append(f"{name}: old action major in {cfg.name}")
        for f in base.rglob("*"):
            if f.is_file() and f.suffix in {".rs", ".toml", ".md", ".yml", ".html"}:
                txt=f.read_text(encoding="utf-8")
                if "{%" in txt or "%}" in txt:
                    errors.append(f"{name}: unsupported Liquid control syntax in {f.relative_to(base)}")
                # Generator placeholder should not leak after simple text render.
                if "{{project-name}}" in txt and "{{project-name}}" in RENDER.sub("smoke-project", txt):
                    errors.append(f"{name}: unrendered project-name in {f.relative_to(base)}")
        if meta["kind"] == "wasm" and "wasm32-unknown-unknown" not in (base / ".github/workflows/ci.yml").read_text():
            errors.append(f"{name}: missing wasm target in CI")
    return errors


def generate_and_check(cargo_check: bool) -> list[str]:
    errors=[]
    if not shutil.which("cargo") or not shutil.which("rustc"):
        return ["Rust toolchain unavailable: cannot run cargo-generate integration checks"]
    if not shutil.which("cargo-generate"):
        return ["cargo-generate unavailable: install with cargo install cargo-generate --locked"]
    with tempfile.TemporaryDirectory(prefix="rust-starter-") as tmp:
        tmpdir=Path(tmp)
        for name, meta in PROFILES.items():
            project=f"smoke-{name.replace('_','-')}"
            try:
                generated = tmpdir / project
                # cargo-generate 0.26 treats --destination as an existing parent
                # directory and creates the project at <destination>/<name>.
                generated.parent.mkdir(parents=True, exist_ok=True)
                run(["cargo", "generate", "--path", str(ROOT / "templates" / name),
                     "--name", project, "--destination", str(generated.parent)], ROOT)
                if not (generated / "Cargo.toml").exists():
                    raise RuntimeError("No Cargo.toml at generated project path")
                if "{{project-name}}" in (generated / "Cargo.toml").read_text():
                    raise RuntimeError("Generator left project-name placeholder")
                run([sys.executable, "scripts/check_agents.py"], generated)
                if cargo_check:
                    run(["cargo", "fmt", "--all", "--", "--check"], generated)
                    if meta["kind"] == "wasm":
                        run(["cargo", "check", "--target", "wasm32-unknown-unknown"], generated)
                    else:
                        run(["cargo", "check", "--workspace", "--all-targets"], generated)
                        run(["cargo", "test", "--workspace"], generated)
                print(f"PASS generated {name}")
            except Exception as exc:
                errors.append(f"{name}: {exc}")
    return errors


def main() -> int:
    parser=argparse.ArgumentParser()
    parser.add_argument("--generate",action="store_true",help="run real cargo-generate integration tests")
    parser.add_argument("--cargo-check",action="store_true",help="also compile and test generated projects")
    args=parser.parse_args()
    if args.cargo_check and not args.generate:
        parser.error("--cargo-check requires --generate")
    errors=check_templates()
    if not errors:
        print(f"PASS offline: {len(PROFILES)} profiles, manifests, workflows and mirrored agent skills")
    if args.generate:
        errors.extend(generate_and_check(args.cargo_check))
    if errors:
        print(f"FAIL: {len(errors)} problem(s)")
        for e in errors: print(" -",e)
        return 1
    print("All requested checks passed")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
