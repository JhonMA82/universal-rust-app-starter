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
        ["History<T>", "Format<T>", "storage::Document<T>", "Document::open", "Document::save"],
        [
            "Choose a stable format ID, version, byte limit and validation function. Implement consecutive migrations when increasing the version.",
            "On native targets create a `storage::Document::new(format, initial, undo_capacity)` or open an existing file. Call `replace(candidate)`, `undo()`, `redo()` and `save(path)` on that session. The session owns the checksum, disk revision and save point.",
            "For a native file, save an `Envelope` with the last disk checksum or `None` for a new file. Keep the returned checksum. Apply `History::mark_saved` only after the verified write succeeds. Add a round-trip and an external-write conflict test.",
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


def write_handbook(project, report, metadata):
    """Write portable, composition-aware project documentation."""
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
        "crates/app/src/domain.rs", "crates/starter-demo-domain/src/lib.rs",
        "crates/starter-notes/src/lib.rs",
    ])
    views = paths_for(project, ["crates/app/src/view.rs", "crates/starter-demo/src/lib.rs"])
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
    product_sources = paths_for(project, [
        "crates/app/src/domain.rs", "crates/app/src/view.rs", "crates/app/src/product.rs",
        "crates/starter-demo-domain/src/lib.rs" if {"data-demo", "engineering-demo"} & modules else "",
        "crates/starter-notes/src/lib.rs" if "documents" in modules else "",
        "crates/starter-demo/src/lib.rs", entry,
    ])
    if "application-core" in modules and not (
        {"document-demo", "data-demo", "engineering-demo"} & modules
        or "crates/app/src/domain.rs" in product_sources
    ):
        product_sources.extend(paths_for(project, ["crates/starter-core/src/sample.rs"]))
    forge = (project / "docs/EGUI_VISUAL_FORGE.md").is_file()
    skills = sorted(relative(project, file) for file in (project / ".agents/skills").glob("*/SKILL.md"))
    inventory = {
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
    lines = [
        "# Develop the product", "",
        f"Profile `{profile}`, preset `{report['preset']}`.", "",
        "## Start", "",
        "Run one entry point at a time from the generated project root. The first command starts the app, server or library test suite.",
        "", "```sh", *launch, "```", "",
    ]
    if installed := inventory["capabilities"]:
        lines.extend(["Installed capabilities " + ", ".join(f"`{value}`" for value in installed) + ".", ""])
    if "desktop-product" in modules:
        lines.extend([
            "Start in `crates/app/src/domain.rs` and `crates/app/src/view.rs`. The desktop product already wires the registry, rendering and native tasks. Run `cargo run -p desktop -- --operations` to inspect the operation IDs.",
            "",
        ])
    if commands := list(integration.guides(profile, interfaces)):
        lines.extend([*["- " + line for line in commands], ""])
    for boundary in boundaries:
        lines.extend([boundary, ""])
    lines.extend([
        "## Make the first domain change", "",
        "Before adding infrastructure, inspect [the installed inventory](../infrastructure.json) and reuse available components and commands. Dependencies specific to your domain remain the product's decision.", "",
        *[f"- [`{source}`](../{source})" for source in product_sources], "",
        "Put the rule in your domain, register it once, then add the view or client that invokes it. Test the rule and operation independently of the interface.", "",
    ])
    for recipe in recipes:
        lines.extend([
            f"### {recipe['name']}", "",
            ", ".join(f"`{api}`" for api in recipe["apis"]) + ".", "",
            *[f"- [`{path}`](../{path})" for path in recipe["paths"]], "",
            *[f"{i}. {step}" for i, step in enumerate(recipe["steps"], 1)], "",
        ])
    if forge:
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
        "```sh", *verification, "```", "",
        *(["Run `cargo xtask --help` to discover the available development commands.", ""]
          if "engineering-tools" in modules else []),
        "[Agent instructions](../AGENTS.md) list the remaining project conventions.",
        *[f"See [`{path}`](../{path})." for path in docs], "",
    ])
    handbook = project / "docs/GETTING_STARTED.md"
    handbook.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    (project / "infrastructure.json").write_text(
        json.dumps(inventory, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n",
    )
