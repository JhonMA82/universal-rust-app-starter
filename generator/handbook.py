"""Produce a self-contained guide and machine inventory from a staged project."""
import json
from pathlib import Path

if __package__:
    from . import integration
else:
    import integration


ENTRY = {
    "library": ("cargo test --workspace", "src/lib.rs"),
    "cli": ("cargo run -- --help", "src/main.rs"),
    "desktop-egui": ("cargo run -p desktop", "crates/app/src/lib.rs"),
    "desktop-egui-web": ("cargo run -p desktop", "crates/app/src/lib.rs"),
    "service-axum": ("cargo run", "src/main.rs"),
    "mcp-stdio": ("cargo run", "src/main.rs"),
    "mcp-http": ("cargo run", "src/main.rs"),
    "mcp-client": ("cargo run", "src/main.rs"),
    "web-leptos-csr": ("trunk serve", "src/main.rs"),
    "web-leptos-fullstack": ("cargo run", "src/main.rs"),
    "web-dioxus": ("dx serve --platform web", "src/main.rs"),
}

PRODUCT_DOMAINS = {
    "desktop-product": "crates/app/src/domain.rs",
    "document-demo": "crates/starter-notes/src/lib.rs",
    "inspection-demo": "crates/starter-inspection/src/lib.rs",
    "data-demo": "crates/starter-demo-domain/src/lib.rs",
    "engineering-demo": "crates/starter-demo-domain/src/lib.rs",
    "lab-workbench": "crates/starter-lab/src/domain.rs",
}


def relative(project, path):
    return Path(path).resolve().relative_to(project.resolve()).as_posix()


def workspace_packages(project, metadata):
    """Cargo metadata is authoritative for crate names, features and dependencies."""
    packages = {}
    members = set(metadata["workspace_members"])
    for package in sorted(metadata["packages"], key=lambda value: value["name"]):
        if package["id"] not in members:
            continue
        dependencies = []
        for dependency in package["dependencies"]:
            item = {
                key: dependency[key]
                for key in ("name", "req", "kind", "target", "features", "optional")
            }
            if dependency.get("path"):
                item["path"] = relative(project, dependency["path"])
            if dependency.get("rename"):
                item["rename"] = dependency["rename"]
            dependencies.append(item)
        packages[package["name"]] = {
            "manifest": relative(project, package["manifest_path"]),
            "version": package["version"],
            "rust_version": package["rust_version"],
            "license": package["license"],
            "features": package["features"],
            "targets": [
                {"name": target["name"], "kind": target["kind"],
                 "source": relative(project, target["src_path"])}
                for target in package["targets"]
            ],
            "dependencies": sorted(dependencies, key=lambda item: json.dumps(item, sort_keys=True)),
        }
    return packages


def module_guide(module, label, paths, apis, steps):
    return {"module": module, "name": label, "paths": paths, "apis": apis, "steps": steps}


RECIPES = [
    module_guide(
        "application-core", "Register one operation",
        ["crates/starter-core/src/lib.rs", "crates/starter-core/src/sample.rs"],
        ["Application::command", "Application::query", "Definition", "Request"],
        [
            "Add a typed input deriving `Deserialize` and `JsonSchema` beside your domain type. Register it once using a stable ID and a permission such as `read` or `write`.",
            "Validate inside the command and return `Error::new(code, details)` on rejection. Successful writes increment the revision and append an event. Pass the observed revision through `Request::expected_revision` from clients that can edit concurrently.",
            "Test success, permission failure, rollback and a stale revision with `Application::invoke`. Reuse the operation ID in your view and discover it through the installed adapters.",
        ],
    ),
    module_guide(
        "reference-runtime", "Add a workspace panel",
        ["crates/starter-reference/src/lib.rs", "crates/app/src/lib.rs"],
        ["starter_reference::View", "starter_reference::launch", "starter_reference::window",
         "starter_reference::invoke", "View::panels", "View::initial_workspace", "View::design", "Panel::Custom"],
        [
            "Implement your project view with `View::show` and `View::status`. Return a stable custom instance from `View::panels` and render just that instance in `show`. Use the existing host rather than creating an eframe application.",
            "Call `starter_reference::invoke` on a shared application from a control. Supply the current document and expected revision when the action depends on them. Treat failures as UI feedback.",
            "Use Commands (Ctrl/Cmd+P) to discover public operations and their input schemas. Use Preferences and the Components tab to check appearance.",
            "Use `View::initial_workspace` to provide an initial `Layout` and `Tokens`. Stored user preferences take precedence.",
            "Customize the product accents by returning a `starter_desktop::Design` from `View::design`. Set `dark_accent` and `light_accent` using `starter_desktop::egui::Color32`. The host applies the design to the shell and gallery.",
        ],
    ),
    module_guide(
        "desktop-shell", "Build a property editor",
        ["crates/starter-desktop/src/components.rs", "crates/starter-desktop/src/design.rs", "docs/modules/desktop-shell.md"],
        ["PanelSpec", "Tokens", "Design", "property_grid", "field", "unit_field", "notice", "progress", "confirmation"],
        [
            "Use standard egui `Button`, `ComboBox`, `DragValue` and `TextEdit`. Associate text inputs with their label using `.labelled_by(label.id)` and group related fields with `starter_desktop::property_grid`. Use `field` or `search_field` for labeled reusable input.",
            'Use `starter_desktop::unit_field(ui, "Width ", " m", &mut width, 0.0..=10.0)` for a dimension, `notice` for a validation error and `progress` for bounded progress. The shell applies its `Tokens` to the egui context.',
            "Show recoverable feedback with `dismissible_notice`, a placeholder with `empty_state`, or a spinner with `loading`.",
            "Give a save/replace action with irreversible effects an explicit confirmation using `starter_desktop::confirmation`. Put the change itself in the registered domain operation.",
        ],
    ),
    module_guide(
        "reference-runtime", "Expose a command in the menu and toolbar",
        ["crates/starter-reference/src/lib.rs", "crates/starter-desktop/src/actions.rs"],
        ["ActionSpec", "View::actions", "View::action_parameters"],
        [
            "Register the operation once. Define a `const ACTIONS: &[ActionSpec]` with its ID, label and optional menu, toolbar placement and keyboard shortcut. Return the slice from `View::actions`.",
            "Build the input for that ID in `View::action_parameters`. The host checks availability and invokes the public registry operation with the active session, document and revision.",
            "Run `cargo test -p starter-reference` for menu, toolbar and keyboard dispatch. Test your domain rule independently of the presentation.",
        ],
    ),
    module_guide(
        "document-demo", "Replace the sample notes model",
        ["crates/starter-notes/src/lib.rs", "crates/starter-demo/src/lib.rs"],
        ["starter_notes::Domain", "starter_notes::format", "View::request_save"],
        [
            "Define your document model and a new format ID. Bind the view to its fields while keeping revision, undo/redo, file conflicts and autosave orchestration.",
            "Commit one change per user action. Leave in-progress drafts in the view. Preserve recovery file paths separately from the user's saved file.",
        ],
    ),
    module_guide(
        "documents", "Save and reopen an application document",
        ["crates/starter-documents/src/lib.rs", "crates/starter-documents/src/storage.rs"],
        ["History<T>", "Format<T>", "storage::Document<T>", "storage::Recovery",
         "Document::open", "Document::save", "Document::autosave"],
        [
            "Choose a stable format ID, version, byte limit and validation function. Implement consecutive migrations when increasing the version.",
            "On native targets create a `storage::Document::new(format, initial, undo_capacity)` or open an existing file. Call `replace(candidate)`, `undo()`, `redo()` and `save(path)` on that session. The session owns the checksum, disk revision and save point.",
            "Use `storage::Recovery` to choose a distinct location for drafts. Call `Document::autosave` with a committed snapshot and use `recover` or `discard_recovery` after the user decides. Add round-trip, recovery and conflict tests.",
        ],
    ),
    module_guide(
        "inspection-demo", "Extend a typed inspection report",
        ["crates/starter-inspection/src/lib.rs", "crates/starter-demo/src/lib.rs", "docs/modules/inspection-demo.md"],
        ["InspectionReport", "InspectionCheck", "format", "inspection.get", "inspection.edit"],
        [
            "Add a report field in `InspectionReport` or a check field in `InspectionCheck`. Update the default and format validation. Supply a deserialization default for a compatible addition or a versioned migration for a breaking format change.",
            "Display the field in the inspection workbench and bind it to the view draft. Commit through `inspection.edit` with the observed document revision so the registry validates and records the edit.",
            "Run `cargo test -p starter-inspection -p starter-demo` for operation validation, undo, save, reopen and recovery. If automation was selected, discover the updated input schema from the installed MCP transport.",
        ],
    ),
    module_guide(
        "data", "Show a filtered table and plot",
        ["crates/starter-dataset/src/lib.rs", "crates/starter-data/src/lib.rs"],
        ["Data::parse", "Data::filter", "Data::export", "starter_data::table", "starter_data::plot"],
        [
            "Set byte and row limits when opening CSV. Filter and sort into row indices once, then pass the same index slice to the table, plot and export.",
            "Keep the display selection and column index in view state. Account for missing or non-finite numbers in your domain calculations.",
        ],
    ),
    module_guide(
        "data-demo", "Add a measurement calculation",
        ["crates/starter-demo-domain/src/lib.rs", "crates/starter-demo/src/lib.rs"],
        ["starter_demo_domain::metrics", "data.import", "data.filter", "data.csv"],
        [
            "Extend the summary with a statistic that matters to your dataset. Use `Data::values` on the same filtered view as the table and plot.",
            "Add a fixture that imports an application-specific CSV, filters its rows, checks a known result and exports the expected subset.",
        ],
    ),
    module_guide(
        "tasks", "Run a cancellable background task",
        ["crates/starter-reference/src/task_panel.rs", "crates/starter-tasks/src/lib.rs"],
        ["Pool::start", "Control::report", "Control::cancelled", "Job::cancel", "Job::poll"],
        [
            "Start the job from a controller using `pool.start(revision, move |control| { … })`. Copy the needed inputs into the closure and check `control.cancelled()` in long loops.",
            "Report progress in basis points between 0 and 10,000. In egui, pass `f32::from(job.progress()) / 10_000.0` to the installed progress component if you selected the desktop shell.",
            "Poll the job with the current revision from your view update. Display cancellation, failure or stale output without applying the result to a newer model.",
        ],
    ),
    module_guide(
        "engineering-demo", "Add a parametric result",
        ["crates/starter-demo-domain/src/lib.rs", "crates/starter-demo/src/lib.rs"],
        ["Dimensions", "Derived", "History::replace", "Viewport::show"],
        [
            "Replace or extend the parameter model and give each dimension an explicit unit. Validate inputs before making history edits.",
            "Add your computed result as a graph node and compare it with a known analytical case. Generate display geometry from the accepted model and edit parameters via the operation registry.",
        ],
    ),
    module_guide(
        "lab-workbench", "Extend a laboratory calculation",
        ["crates/starter-lab/src/domain.rs", "crates/starter-lab/src/view.rs", "crates/starter-lab/src/lib.rs"],
        ["Tank", "evaluate", "tank.parameters", "tank.resize", "tank.litres"],
        [
            "Work in `Tank` and its validation. Keep the persisted parameters and units in the typed document, and render the same committed values in the canvas and viewport.",
            "Use the evaluation graph for derived results and the existing background task API for long calculations. Publish a result with the revision that started the work to keep stale results out of the model.",
            "Use `cargo test -p starter-lab` to verify an analytical result and cancellation. For an enabled MCP adapter, discover and invoke the public registry operations. Keep file actions and result publication in local commands.",
        ],
    ),
    module_guide(
        "evaluation", "Cache a derived result",
        ["crates/starter-evaluation/src/lib.rs"],
        ["Graph::add", "Graph::depend", "Graph::invalidate", "Graph::evaluate"],
        [
            "Declare inputs and results as graph nodes. Invalidate only changed sources and evaluate with a cancellation predicate.",
            "Check one known calculation plus the number of nodes affected by an edit. Failed evaluations must leave the previous valid cache usable.",
        ],
    ),
    module_guide(
        "science", "Verify physical units",
        ["crates/starter-science/src/lib.rs"],
        ["quantities", "Tolerance::new", "ReferenceCase"],
        [
            "Use dimensioned quantities in algorithms and convert units only at UI or file boundaries. Configure the numeric error budget and test expected results before exposing them to a view.",
        ],
    ),
    module_guide(
        "canvas", "Add a 2D editor",
        ["crates/starter-canvas/src/lib.rs"],
        ["Camera::interact", "Camera::screen", "Camera::world", "Camera::snap"],
        [
            "Convert the pointer from screen to world coordinates before selecting or snapping. Keep the camera in the view and commit only accepted model changes through the operation registry.",
        ],
    ),
    module_guide(
        "viewport", "Render a 3D result",
        ["crates/starter-viewport-core/src/lib.rs", "crates/starter-viewport-egui/src/lib.rs"],
        ["Scene", "Mesh", "Scene::pick", "Viewport::new", "Viewport::show"],
        [
            "Build a renderer-neutral scene from the model. Construct the egui adapter with eframe's wgpu render state and reuse it while the scene changes.",
            "Test picking and camera transforms on the CPU. Use the GPU capture test when an adapter is available.",
        ],
    ),
    module_guide(
        "application-mcp", "Expose a domain operation through MCP",
        ["docs/modules/application-mcp.md"],
        ["starter_mcp::Server", "capabilities", "execute"],
        [
            "Register a public operation in the same engine used by the application. Require the right permission and return a stable structured error. Leave internal operations private.",
            "Run the installed transport from the entry points above. Discover the operation and its schema with `capabilities`, then call `execute` with the discovered session ID and current revision.",
        ],
    ),
    module_guide(
        "application-http", "Expose an operation over HTTP",
        ["docs/modules/application-http.md"],
        ["starter_http::router", "GET /operations", "POST /invoke"],
        [
            "Register a public operation and use the installed router. Configure a unique `APP_OPERATIONS_TOKEN` at runtime and pass it as an Authorization bearer credential.",
        ],
    ),
]


def paths_for(project, patterns):
    return [name for name in patterns if (project / name).is_file()]


def build_inventory(project, report, metadata):
    """Derive project documentation data from the installed composition."""
    profile = report["profile"]
    modules = set(report["modules"])
    start, entry = ENTRY[profile]
    interfaces = report["entrypoints"]
    platforms = report.get("profile_targets", report["targets"])
    verification = ["cargo fmt --all -- --check"]
    if "native" in report["targets"]:
        verification.append("cargo test --workspace")
    if "wasm" in report["targets"]:
        package = report.get("wasm_package")
        selection = f" -p {package}" if package else ""
        verification.append(f"cargo check{selection} --target wasm32-unknown-unknown")
    if "engineering-tools" in modules:
        verification.append("cargo xtask verify")
    verification.append("python3 scripts/check_agents.py")

    if profile == "desktop-egui-web":
        launch = [start, "trunk serve"]
    else:
        launch = [start]
    if "operations-cli" in interfaces:
        launch.append(f"{start if profile.startswith('desktop-egui') else 'cargo run'} -- --operations")
    elif "operations-cli-companion" in interfaces:
        launch.append("cargo run --bin operations -- --operations")

    domains = paths_for(project, [
        source for module, source in PRODUCT_DOMAINS.items() if module in modules
    ])
    if not domains and "documents" in modules and profile == "desktop-egui-web":
        domains = paths_for(project, ["crates/starter-notes/src/lib.rs"])
    views = paths_for(project, [
        "crates/app/src/view.rs", "crates/starter-demo/src/lib.rs",
        "crates/starter-lab/src/view.rs", "crates/starter-lab/src/lib.rs",
    ])
    recipes = []
    for recipe in RECIPES:
        module = recipe["module"]
        if module not in modules:
            continue
        paths = paths_for(project, recipe["paths"])
        if module == "application-core" and domains:
            paths = [*domains, "crates/starter-core/src/lib.rs"]
        elif module == "reference-runtime":
            paths = [*views, *paths]
        recipes.append(recipe | {"paths": paths})
    boundaries = []
    if "documents" in modules and "wasm" in platforms:
        boundaries.append("Browser documents use in-memory history. The shipped filesystem storage and recovery paths are native-only.")
    if "application-mcp" in modules:
        boundaries.append("Use the selected local MCP transport and the declared trust model. Development UI inspection is a separate opt-in endpoint.")
    if "viewport" in modules:
        boundaries.append("The 3D viewport needs a native wgpu adapter. Offscreen capture is a blocking evidence/export operation.")
    if profile == "mcp-client":
        boundaries.append("The MCP client needs a running local MCP server. Set MCP_URL for a different approved endpoint.")
    if "wasm" in platforms:
        boundaries.append("Install wasm32-unknown-unknown with rustup and use the browser tool named by the profile.")
    docs = ["docs/COMPOSITION.md", "docs/ARCHITECTURE.md"] if modules else ["docs/ARCHITECTURE.md"]
    docs = paths_for(project, docs)
    docs.extend(sorted(relative(project, file) for file in (project / "docs/modules").glob("*.md")))
    product_sources = list(dict.fromkeys([*domains, *views, *paths_for(project, [entry])]))
    if "application-core" in modules and not domains:
        product_sources.extend(paths_for(project, ["crates/starter-core/src/sample.rs"]))
    forge = (project / "docs/EGUI_VISUAL_FORGE.md").is_file()
    skills = sorted(relative(project, file) for file in (project / ".agents/skills").glob("*/SKILL.md"))
    return {
        "schema_version": 1,
        "profile": profile,
        "preset": report["preset"],
        "product": report.get("product"),
        "targets": report["targets"],
        "profile_targets": platforms,
        "requested_features": report.get("features", []),
        "modules": report["modules"],
        "capabilities": report.get("capabilities", []),
        "entrypoints": interfaces,
        "crates": workspace_packages(project, metadata),
        "skills": skills,
        "docs": docs,
        "product_sources": product_sources,
        "launch": launch,
        "verification": verification,
        "recipes": recipes,
        "restrictions": boundaries,
        "visual_forge": "docs/EGUI_VISUAL_FORGE.md" if forge else None,
    }


def render_handbook(inventory):
    profile = inventory["profile"]
    modules = inventory["modules"]
    lines = [
        "# Develop the product", "",
        f"Profile `{profile}`, preset `{inventory['preset']}`.", "",
        "## Start", "",
        "Run one entry point at a time from the generated project root. The first command starts the app, server or library test suite.",
        "", "```sh", *inventory["launch"], "```", "",
    ]
    if installed := inventory["capabilities"]:
        lines.extend(["Installed capabilities " + ", ".join(f"`{value}`" for value in installed) + ".", ""])
    if "desktop-product" in modules:
        lines.extend([
            "Start in `crates/app/src/domain.rs` and `crates/app/src/view.rs`. The desktop product already wires the registry, rendering and native tasks. Run `cargo run -p desktop -- --operations` to inspect the operation IDs.",
            "",
        ])
    if "inspection-demo" in modules:
        lines.extend([
            "Use Apply changes to validate and commit the editor draft. Enter a file path to save or reopen the report. Use the Recovery panel to review available autosaved drafts.",
            "",
        ])
    if commands := list(integration.guides(profile, inventory["entrypoints"])):
        lines.extend([*["- " + line for line in commands], ""])
    for boundary in inventory["restrictions"]:
        lines.extend([boundary, ""])
    lines.extend([
        "## Make the first domain change", "",
        "Before adding infrastructure, inspect [the installed inventory](../infrastructure.json) and reuse available components and commands. Dependencies specific to your domain remain the product's decision.", "",
        *[f"- [`{source}`](../{source})" for source in inventory["product_sources"]], "",
        "Put the rule in your domain, register it once, then add the view or client that invokes it. Test the rule and operation independently of the interface.", "",
    ])
    for recipe in inventory["recipes"]:
        lines.extend([
            f"### {recipe['name']}", "",
            ", ".join(f"`{api}`" for api in recipe["apis"]) + ".", "",
            *[f"- [`{path}`](../{path})" for path in recipe["paths"]], "",
            *[f"{i}. {step}" for i, step in enumerate(recipe["steps"], 1)], "",
        ])
    if inventory["visual_forge"]:
        lines.extend([
            "## Inspect and compare the interface", "",
            "[Visual Forge](EGUI_VISUAL_FORGE.md) documents the local environment, screenshot capture, scene files and comparison. Install the tools in that environment with `uv sync` and run its checks with `uv run python tools/validate_toolkit.py`.",
            "",
        ])
        if "reference-runtime" in modules:
            lines.extend([
                "For an explicit local development session, run `cargo run -p desktop --features ui-inspection -- --ui-inspect` and attach the bridge described in [the runtime guide](modules/reference-runtime.md).",
                "",
            ])
        lines.extend([
            "Capture the standard and changed states at the same size, scale, theme and density. Given `reference.png` and `candidate.png`, create a scene and compare their pixels.",
            "", "```sh",
            "uv run python tools/init_scene.py --image reference.png --out scene.json --pixels-per-point 1",
            "uv run python tools/visual_check.py --scene scene.json --candidate candidate.png --out qa/iteration-01",
            "```", "",
        ])
    lines.extend([
        "## Installed crates", "",
        "| Local crate | Version | Manifest |",
        "|---|---|---|",
        *[f"| `{name}` | {crate['version']} | [Cargo.toml](../{crate['manifest']}) |"
          for name, crate in inventory["crates"].items()],
        "",
        "## Verify", "",
        "```sh", *inventory["verification"], "```", "",
        *(["Run `cargo xtask --help` to discover the available development commands.", ""]
          if "engineering-tools" in modules else []),
        "[Agent instructions](../AGENTS.md) list the remaining project conventions.",
        *[f"See [`{path}`](../{path})." for path in inventory["docs"]], "",
    ])
    if inventory["skills"]:
        lines.extend([
            "## Installed skills", "",
            *[f"- [`{Path(path).parent.name}`](../{path})" for path in inventory["skills"]],
            "",
        ])
    return "\n".join(lines)



def installed_skills_line(paths):
    links = [f"[`{Path(path).parent.name}`](../{path})" for path in paths]
    return f"Installed ({len(links)}): " + ", ".join(links) + "."

def write_handbook(project, report, metadata):
    """Write the guide and inventory from the same installed-project data."""
    inventory = build_inventory(project, report, metadata)
    (project / "docs/GETTING_STARTED.md").write_text(
        render_handbook(inventory), encoding="utf-8", newline="\n",
    )
    skills_doc = project / "docs/SKILLS.md"
    if skills_doc.is_file():
        lines = skills_doc.read_text(encoding="utf-8").splitlines()
        matches = [index for index, line in enumerate(lines) if line.startswith("Installed (")]
        if len(matches) != 1:
            raise ValueError("expected one installed skills list in docs/SKILLS.md")
        lines[matches[0]] = installed_skills_line(inventory["skills"])
        skills_doc.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    (project / "infrastructure.json").write_text(
        json.dumps(inventory, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n",
    )
