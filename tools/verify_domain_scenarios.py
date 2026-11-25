#!/usr/bin/env python3
"""Exercise project-owned features in four ordinary composed native applications."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from generator.compose import generate
from starter import plan
from tools.verify_composed import cli, mcp


def edit(project, name, before, after):
    source = project / name
    content = source.read_text(encoding="utf-8")
    if content.count(before) != 1:
        raise ValueError(f"{name}: expected one occurrence of {before!r}")
    source.write_text(content.replace(before, after, 1), encoding="utf-8", newline="\n")


def desktop_extension(project):
    edit(project, "crates/app/src/domain.rs", "    app\n}", """    app.query(
        operation("workspace.name_length", "Count the workspace name characters", "read"),
        |state, _: Empty, _| Ok(state.name.chars().count()),
    )
    .expect("unique operation");
    app
}""")
    edit(project, "crates/app/src/view.rs", '            ui.label(format!("Domain revision {}", current.revision));', '''
            ui.label(format!("Domain revision {}", current.revision));
            if let Ok(length) = invoke(&self.app, "workspace.name_length", json!({}), None, None) {
                ui.label(format!("Name length: {}", length.result));
            }'''.lstrip("\n"))
    return [("workspace.name_length", {}, None, 9)]


CHECKLIST = r'''
use eframe::egui;
use starter_documents::{Envelope, Error, Format, History, storage::{self, Checksum}};
use std::path::Path;

type Checklist = Vec<(String, bool)>;

const FORMAT: Format<Checklist> = Format {
    id: "product-checklist",
    version: 1,
    max_bytes: 16 * 1024,
    migrate: |_, _| Err(Error::Format),
    validate: |items| {
        if items.len() > 100 || items.iter().any(|(title, _)| title.trim().is_empty() || title.len() > 128) {
            return Err(Error::Invalid("Use 1–128 bytes per item and up to 100 items".into()));
        }
        Ok(())
    },
};

pub struct Editor {
    history: History<Checklist>,
    checksum: Option<Checksum>,
    message: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self { history: History::new(vec![("Confirm requirements".into(), false)], 32), checksum: None, message: String::new() }
    }
}

impl Editor {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Product checklist");
        let mut candidate = self.history.value().clone();
        let mut changed = false;
        for (label, checked) in &mut candidate {
            changed |= ui.checkbox(checked, label.as_str()).changed();
        }
        if changed {
            self.message = match self.history.replace(candidate, FORMAT.validate) {
                Ok(_) => "Edit committed".into(),
                Err(error) => error.to_string(),
            };
        }
        if ui.button("Undo checklist change").clicked() {
            self.message = match self.history.undo() {
                Ok(_) => "Undo complete".into(),
                Err(error) => error.to_string(),
            };
        }
        let path = Path::new("product-checklist.json");
        if ui.button("Save product-checklist.json").clicked() {
            self.message = match self.save(path) {
                Ok(()) => "Checklist saved".into(),
                Err(error) => error.to_string(),
            };
        }
        if ui.button("Reopen product-checklist.json").clicked() {
            self.message = match self.open(path) {
                Ok(()) => "Checklist opened".into(),
                Err(error) => error.to_string(),
            };
        }
        starter_desktop::notice(ui, &self.message, starter_desktop::Tone::Normal);
    }

    fn save(&mut self, path: &Path) -> Result<(), Error> {
        let snapshot = self.history.snapshot();
        let value = Envelope {
            format: FORMAT.id.into(),
            version: FORMAT.version,
            revision: self.history.revision(),
            content: (*snapshot).clone(),
            extra: Default::default(),
        };
        self.checksum = Some(storage::save(path, &FORMAT, &value, self.checksum)?.checksum);
        self.history.mark_saved(snapshot);
        Ok(())
    }

    fn open(&mut self, path: &Path) -> Result<(), Error> {
        let (doc, checksum) = storage::open(path, &FORMAT)?;
        self.history = History::new(doc.content, 32);
        self.checksum = Some(checksum);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_round_trip_uses_generated_history_and_storage() {
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("product-checklist.json");
        let mut editor = Editor::default();
        let done = vec![("Confirm requirements".to_owned(), true)];
        editor.history.replace(done.clone(), FORMAT.validate).unwrap();
        assert!(editor.history.undo().unwrap());
        assert!(!editor.history.value()[0].1);
        assert!(editor.history.redo().unwrap());
        editor.save(&path).unwrap();
        assert!(!editor.history.dirty());
        let mut reopened = Editor::default();
        reopened.open(&path).unwrap();
        assert_eq!(reopened.history.value(), &done);
        assert!(matches!(Editor::default().save(&path), Err(Error::Conflict)));
    }
}
'''


def document_extension(project):
    name = "crates/starter-demo/src/lib.rs"
    (project / "crates/starter-demo/src/checklist.rs").write_text(CHECKLIST.lstrip(), encoding="utf-8", newline="\n")
    edit(project, name, "use eframe::egui;", "mod checklist;\nuse eframe::egui;")
    edit(project, name, "struct Editor {", "struct Editor {\n    checklist: checklist::Editor,")
    edit(project, name, "        app,\n        current: 1,", "        checklist: checklist::Editor::default(),\n        app,\n        current: 1,")
    edit(project, name, "            app: Arc::new", "            checklist: checklist::Editor::default(),\n            app: Arc::new")
    edit(project, name, "static PANELS: [starter_desktop::PanelSpec; 1] = [starter_desktop::PanelSpec {", """
        static PANELS: [starter_desktop::PanelSpec; 2] = [starter_desktop::PanelSpec {
            instance: 102,
            kind: "product.checklist",
            title: "Checklist",
            floating: true,
            initially_open: false,
        }, starter_desktop::PanelSpec {""".strip())
    edit(project, name, "        if panel == Panel::Custom(100) {", """
        if panel == Panel::Custom(102) {
            self.checklist.show(ui);
            return;
        }
        if panel == Panel::Custom(100) {""".lstrip("\n"))
    return []


def data_extension(project):
    name = "crates/starter-demo-domain/src/lib.rs"
    edit(project, name, "    pub mean: Option<f64>,", "    pub mean: Option<f64>,\n    pub peak: Option<f64>,")
    edit(project, name, "    let (mut mean, mut n) = (0.0, 0);", "    let (mut mean, mut n) = (0.0, 0);\n    let mut peak: Option<f64> = None;")
    edit(project, name, "        n += 1;", "        n += 1;\n        peak = Some(peak.map_or(value, |current| current.max(value)));")
    edit(project, name, "        rows: view.indices.len(),", "        peak,\n        rows: view.indices.len(),")
    edit(project, "crates/starter-demo/src/lib.rs", 'format!("{} numeric rows · mean {:?}", s.numeric, s.mean)', 'format!("{} numeric rows · mean {:?} · peak {:?}", s.numeric, s.mean, s.peak)')
    csv = "instrument,load_n\npress-a,20\npress-b,80\npress-a,100\n"
    return [
        ("data.import", {"csv": csv}, None, 3),
        ("data.filter", {"query": "press-a", "column": 1, "sort": 1}, None, 2),
        ("data.summary", {}, None, {"rows": 2, "mean": 60.0, "peak": 100.0}),
        ("data.csv", {}, None, "instrument,load_n\npress-a,100\npress-a,20\n"),
    ]


def engineering_extension(project):
    edit(project, "crates/starter-demo-domain/src/lib.rs", "    a.query(\n        definition(\"box.volume", '''
    a.query(
        definition("floor.area", "Compute footprint in square metres", "read"),
        |history, _: Empty, _| {
            let dimensions = history.value();
            Ok(dimensions.width_m * dimensions.depth_m)
        },
    )
    .unwrap();
    a.query(
        definition("box.volume'''.lstrip("\n"))
    edit(project, "crates/starter-demo/src/lib.rs", '            ui.label(format!("Volume: {:.6} m³", self.metrics.1));', '''
            ui.label(format!("Volume: {:.6} m³", self.metrics.1));
            if let Ok(area) = invoke(&self.app, "floor.area", json!({}), None, None) {
                ui.label(format!("Floor area: {} m²", area.result));
            }'''.lstrip("\n"))
    return [
        ("box.resize", {"width_m": 3, "height_m": 2, "depth_m": 6}, None, {"width_m": 3}),
        ("floor.area", {}, None, 18.0),
        ("box.undo", {}, None, {"width_m": 2}),
        ("floor.area", {}, None, 4.0),
    ]


SCENARIOS = {
    "desktop": ("desktop-application", [], ["ui_app", "starter-desktop", "starter-tasks"], desktop_extension),
    "documents": ("document-workbench", [], ["starter-demo", "starter-documents"], document_extension),
    "data": ("data-workbench", ["automation"], ["starter-demo", "starter-tasks", "starter-data"], data_extension),
    "engineering": ("engineering-workbench", ["automation"], ["starter-demo-domain", "starter-viewport-core", "starter-evaluation"], engineering_extension),
}


def run(name, parent, outcome):
    product, capabilities, packages, extend = SCENARIOS[name]
    project = parent / name
    begin = time.monotonic()
    preview = plan({"product": product, "capabilities": capabilities, "name": f"domain-{name}"})
    report = generate(**preview["composition"], destination=project,
                      product=product, capabilities=preview["capabilities"])
    outcome["generate_ms"] = round((time.monotonic() - begin) * 1000)
    inventory = json.loads((project / "infrastructure.json").read_text(encoding="utf-8"))
    assert set(packages) <= set(inventory["crates"])
    assert (project / "docs/GETTING_STARTED.md").is_file()
    original = {
        (project / crate["manifest"]): (project / crate["manifest"]).read_bytes()
        for crate in inventory["crates"].values()
    }
    begin = time.monotonic()
    scenario = extend(project)
    assert all(path.read_bytes() == manifest for path, manifest in original.items())
    outcome["extension_ms"] = round((time.monotonic() - begin) * 1000)
    outcome["modified_manifests"] = []
    outcome["modules"] = list(report["modules"])
    subprocess.run(["cargo", "fmt", "--all"], cwd=project, check=True)
    subprocess.run(["cargo", "test", *[option for package in packages for option in ("-p", package)]],
                   cwd=project, check=True)
    if scenario:
        binary = project / ("domain-probe.exe" if os.name == "nt" else "domain-probe")
        build = subprocess.run(["cargo", "build", "-p", "desktop", "--message-format=json-render-diagnostics"],
                               cwd=project, stdout=subprocess.PIPE, check=True, text=True)
        executable, = [
            row["executable"] for line in build.stdout.splitlines()
            if (row := json.loads(line)).get("reason") == "compiler-artifact"
            and row["target"]["kind"] == ["bin"] and row["target"]["name"] == "desktop"
        ]
        # Each scenario has its own executable even when Cargo reuses a shared target directory.
        shutil.copy2(executable, binary)
        if "mcp-stdio" in report["entrypoints"]:
            mcp(binary, project, scenario)
        else:
            cli(binary, project, scenario)
    outcome["verified_ms"] = round((time.monotonic() - begin) * 1000)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", action="append", choices=SCENARIOS)
    parser.add_argument("--evidence", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    evidence = {
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "source_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=root)),
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "mode": "cargo tests and real CLI/MCP processes without graphical window",
        "scenarios": {},
    }
    with tempfile.TemporaryDirectory(prefix="starter-domain-") as work:
        for name in args.scenario or SCENARIOS:
            result = evidence["scenarios"][name] = {}
            try:
                run(name, Path(work), result)
                result["status"] = "passed"
            except (AssertionError, ValueError, OSError, subprocess.CalledProcessError) as error:
                result["status"] = "failed"
                result["error"] = str(error)
            finally:
                args.evidence.parent.mkdir(parents=True, exist_ok=True)
                args.evidence.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    return any(row["status"] != "passed" for row in evidence["scenarios"].values())


if __name__ == "__main__":
    sys.exit(main())
