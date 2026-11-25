#!/usr/bin/env python3
"""Validate skill metadata, local references, Python syntax and bundled scene files."""
from __future__ import annotations

import argparse
import ast
import re
import sys
from pathlib import Path

import yaml

SKILL_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
LOCAL_REF_RE = re.compile(r"(?:references|scripts|assets)/[A-Za-z0-9_./-]+")
FORGE_SKILLS = (
    "egui-reconstruction",
    "egui-runtime-verification",
    "egui-scene-analysis",
    "egui-ui-engineering",
    "egui-visual-optimization",
)


def validate(root: Path) -> list[str]:
    problems: list[str] = []
    skills_root = root / ".agents" / "skills"
    files = [skills_root / name / "SKILL.md" for name in FORGE_SKILLS]
    missing = [path for path in files if not path.is_file()]
    problems.extend(f"Missing forge skill {path}" for path in missing)
    files = [path for path in files if path.is_file()]
    if not files:
        return problems
    used: set[str] = set()
    for skill in files:
        body = skill.read_text(encoding="utf-8")
        parts = body.split("---", 2)
        if len(parts) != 3 or parts[0].strip():
            problems.append(f"{skill}: missing initial YAML frontmatter")
            continue
        try:
            meta = yaml.safe_load(parts[1])
        except yaml.YAMLError as exc:
            problems.append(f"{skill}: invalid YAML {exc}")
            continue
        if not isinstance(meta, dict):
            problems.append(f"{skill}: metadata must be a map")
            continue
        name = meta.get("name")
        description = meta.get("description")
        if not isinstance(name, str) or not SKILL_RE.fullmatch(name) or len(name) > 64:
            problems.append(f"{skill}: invalid name {name!r}")
        elif name != skill.parent.name:
            problems.append(f"{skill}: name must match folder {skill.parent.name}")
        elif name in used:
            problems.append(f"{skill}: duplicate name {name}")
        else:
            used.add(name)
        if not isinstance(description, str) or not 1 <= len(description) <= 1024:
            problems.append(f"{skill}: invalid description length")
        if not parts[2].strip():
            problems.append(f"{skill}: empty instructions")
        if "##" not in parts[2]:
            problems.append(f"{skill}: no structured section headings")
        # Require local skill resources cited by path to actually exist.
        for found in sorted(set(LOCAL_REF_RE.findall(parts[2]))):
            path = skill.parent / found.rstrip(".,):`")
            if not path.is_file():
                problems.append(f"{skill}: referenced resource does not exist: {found}")
        for ref in skill.parent.rglob("*.md"):
            if ref.stat().st_size == 0:
                problems.append(f"{ref}: empty documentation")
    for source in sorted((root / "tools").glob("*.py")):
        try:
            ast.parse(source.read_text(encoding="utf-8"), filename=str(source))
        except SyntaxError as exc:
            problems.append(f"{source}: Python syntax error: {exc}")
    expected = ["README.md", "pyproject.toml", "docs/SCENE_FORMAT.md", "docs/RESEARCH.md",
                "tools/visual_check.py", "tools/init_scene.py", "tools/evaluate_runs.py",
                "tests/test_tools.py", "examples/eframe-starter/Cargo.toml",
                "LICENSE-EGUI-VISUAL-FORGE"]
    for rel in expected:
        if not (root / rel).is_file():
            problems.append(f"Missing required file {rel}")
    # Semantic checks: the validator doesn't falsely call metadata-only success a full test.
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    problems = validate(args.root.resolve())
    if problems:
        print(f"FAILED ({len(problems)} problems):")
        for item in problems:
            print(f" - {item}")
        return 1
    print("PASS: skill YAML, required files, local resource links, nonempty docs and Python syntax")
    print("NOTE: Rust compilation and end-to-end screenshot fidelity require separate tests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
