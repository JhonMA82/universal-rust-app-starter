"""Package a generated workspace using its resolved lockfile."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import zipfile


def cargo(*args):
    return subprocess.run(["cargo", *args], capture_output=True, text=True, check=True).stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("artifacts"))
    parser.add_argument("--web-dist", type=Path)
    args = parser.parse_args()
    report = json.loads(Path("generation-report.json").read_text(encoding="utf-8"))
    metadata = json.loads(cargo("metadata", "--locked", "--format-version=1"))
    files = {}
    if args.web_dist:
        root = args.web_dist.resolve()
        if not (root / "index.html").is_file() or not any(root.rglob("*.wasm")):
            parser.error("browser distribution requires index.html and a WASM artifact")
        for file in root.rglob("*"):
            if file.is_symlink() or not file.resolve().is_relative_to(root):
                parser.error("the browser distribution contains a symbolic link")
            if file.is_file():
                files[f"web/{file.relative_to(root).as_posix()}"] = file
        target = "web"
    else:
        if "native" not in report["profile_targets"]:
            parser.error("web-only profile requires --web-dist")
        command = ["build", "--locked", "--release", "--message-format=json"]
        if report["profile"].startswith("desktop-egui"):
            command.extend(["-p", "desktop"])
        members = set(metadata["workspace_members"])
        for line in cargo(*command).splitlines():
            entry = json.loads(line)
            if (entry.get("reason") == "compiler-artifact" and entry["package_id"] in members
                and entry.get("executable") and entry["target"]["name"] != "xtask"):
                binary = Path(entry["executable"])
                files[f"bin/{binary.name}"] = binary
        if not files and report["profile"] == "library":
            for pattern in ("Cargo.toml", "README.md", "src/**/*.rs", "crates/**/Cargo.toml", "crates/**/src/**/*.rs"):
                files.update({f"source/{file.as_posix()}": file for file in Path().glob(pattern) if file.is_file()})
        if not files:
            parser.error("no distributable product")
        target = subprocess.check_output(["rustc", "-vV"], text=True).split("host: ", 1)[1].splitlines()[0]
    for package in metadata["packages"]:
        if package["name"] == "epaint_default_fonts":
            font_dir = Path(package["manifest_path"]).parent / "fonts"
            notices = list(font_dir.glob("*.txt"))
            if not notices:
                parser.error("failed to locate egui font license notices")
            files.update({f"third-party/{package['name']}-{package['version']}/{notice.name}": notice for notice in notices})
    files.update({file.name: file for file in Path().glob("LICENSE*") if file.is_file()})
    files["Cargo.lock"] = Path("Cargo.lock")
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / f"{report['name']}-{target}.zip"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
        for name, path in sorted(files.items()):
            bundle.write(path, name)
        bundle.writestr("dependencies.json", json.dumps(
            [{key: package.get(key) for key in ("name", "version", "license", "repository")}
             for package in metadata["packages"]], indent=2))
    with archive.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{digest}  {archive.name}\n", encoding="ascii")
    print(archive)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.stderr or str(error))
