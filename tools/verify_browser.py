#!/usr/bin/env python3
"""Build generated browser apps and verify real interactions with Playwright."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
from PIL import Image
import platform
import subprocess
import sys
import tempfile
import threading
import time

from playwright.sync_api import Error as BrowserError, expect, sync_playwright
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from generator.compose import generate
from tools.verify_composed import identity

CASES = {
    "web-leptos-csr": ("application", Path(".")),
    "desktop-egui-web": ("workbench", Path("crates/web")),
    "web-dioxus": ("application", Path(".")),
}


def build(profile, work):
    preset, entry = CASES[profile]
    project = work / profile
    name = f"browser-{profile}"
    generate(profile, preset, name, project)
    dist = work / f"{profile}-dist"
    if profile == "web-dioxus":
        subprocess.run(["dx", "build", "--platform", "web"], cwd=project, check=True)
        cargo_target = Path(os.environ.get("CARGO_TARGET_DIR", project / "target"))
        dist = cargo_target / f"dx/{name}/debug/web/public"
    else:
        subprocess.run(["trunk", "build", "--dist", str(dist)], cwd=project / entry, check=True)
    if not (dist / "index.html").is_file():
        raise RuntimeError(f"missing browser distribution: {dist}")
    return dist


def verify_page(browser, profile, dist, screenshot):
    handler = partial(SimpleHTTPRequestHandler, directory=dist)
    with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        page = browser.new_page()
        try:
            errors = []
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.goto(f"http://127.0.0.1:{server.server_address[1]}")
            if profile == "desktop-egui-web":
                page.locator("html[data-starter-ready=true]").wait_for(state="attached")
                canvas = page.locator("canvas")
                canvas.wait_for(state="visible")
                status = page.get_by_role("status")
                expect(status).to_have_text("")
                canvas.click(position={"x": 3, "y": 3})
                for note in (2, 3):
                    page.keyboard.press("Alt+n")
                    expect(status).to_have_text(f"Note {note} created")
            else:
                page.get_by_label("Workspace name").fill("Browser verification")
                page.get_by_role("button", name="Rename workspace").click()
                page.get_by_role("heading", name="Browser verification").wait_for()
            assert not errors, errors
            if profile == "desktop-egui-web":
                canvas.screenshot(path=screenshot)
            else:
                page.screenshot(path=screenshot)
        finally:
            page.close()
            server.shutdown()
            thread.join()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--screenshots", type=Path, required=True)
    parser.add_argument("--chromium", type=Path, help="use an installed Chromium executable")
    parser.add_argument("--profile", action="append", choices=CASES, help="limit the browser cases")
    args = parser.parse_args()
    args.screenshots.mkdir(parents=True, exist_ok=True)
    report = {
        "source": identity(), "platform": platform.platform(), "target": "wasm32-unknown-unknown",
        "command": [sys.executable, *sys.argv], "checks": [],
    }
    try:
        report["toolchain"] = subprocess.check_output(["rustc", "-vV"], text=True).strip()
        with tempfile.TemporaryDirectory() as temp, sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=args.chromium, args=[
                "--enable-unsafe-swiftshader", "--use-angle=swiftshader",
            ])
            report["browser"] = browser.version
            try:
                for profile in args.profile or CASES:
                    screenshot = args.screenshots / f"{profile}.png"
                    start = time.monotonic()
                    result = {"profile": profile, "status": "passed"}
                    try:
                        verify_page(browser, profile, build(profile, Path(temp)), screenshot)
                        result["interaction"] = "passed"
                        result["artifact"] = screenshot.name
                        with Image.open(screenshot) as image:
                            if all(low == high for low, high in image.convert("RGB").getextrema()):
                                raise RuntimeError("browser capture is a single color; rendering was not verified")
                        result["rendering"] = "passed"
                    except FileNotFoundError as exc:
                        result.update(status="blocked", message=str(exc))
                    except (AssertionError, BrowserError, OSError, RuntimeError, subprocess.CalledProcessError) as exc:
                        result.update(status="failed", message=str(exc))
                    result["duration_ms"] = round((time.monotonic() - start) * 1000)
                    report["checks"].append(result)
                    print(result, flush=True)
            finally:
                browser.close()
    except (FileNotFoundError, BrowserError, subprocess.CalledProcessError) as exc:
        report["checks"].append({"status": "blocked", "message": str(exc)})
    finally:
        (args.screenshots / "browser-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return int(any(check["status"] != "passed" for check in report["checks"]))


if __name__ == "__main__":
    sys.exit(main())
