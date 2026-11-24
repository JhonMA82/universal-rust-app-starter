#!/usr/bin/env python3
"""Build generated browser apps and verify the real page with Playwright."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
import sys
import tempfile
import threading

from playwright.sync_api import sync_playwright
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from generator.compose import generate


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--screenshots", type=Path, required=True)
    args = parser.parse_args()
    args.screenshots.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as temp, sync_playwright() as playwright:
        browser = playwright.chromium.launch(args=["--enable-unsafe-webgpu", "--use-angle=swiftshader"])
        work = Path(temp)
        for profile, preset, entry in [
            ("web-leptos-csr", "application", Path(".")),
            ("desktop-egui-web", "workbench", Path("crates/web")),
        ]:
            project = work / profile
            generate(profile, preset, f"web-smoke-{preset}", project)
            dist = work / f"{preset}-dist"
            subprocess.run(["trunk","build","--dist",str(dist)], cwd=project / entry, check=True)
            handler = partial(SimpleHTTPRequestHandler, directory=dist)
            with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
                thread = threading.Thread(target=server.serve_forever, daemon=True)
                thread.start()
                try:
                    page = browser.new_page()
                    errors = []
                    page.on("pageerror", lambda error: errors.append(str(error)))
                    page.goto(f"http://127.0.0.1:{server.server_address[1]}")
                    if profile == "web-leptos-csr":
                        page.get_by_label("Workspace name").fill("Browser verification")
                        page.get_by_role("button", name="Rename workspace").click()
                        page.get_by_role("heading", name="Browser verification").wait_for()
                    else:
                        page.locator("html[data-starter-ready=true]").wait_for(state="attached")
                        page.locator("canvas").wait_for(state="visible")
                    page.screenshot(path=args.screenshots / f"{profile}.png")
                    assert not errors, errors
                    page.close()
                finally:
                    server.shutdown()
                    thread.join()
        browser.close()


if __name__ == "__main__":
    main()
