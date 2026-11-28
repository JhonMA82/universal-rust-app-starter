"""Compose the entrypoints for executable capabilities."""
from pathlib import Path

DESKTOP = {"desktop-egui", "desktop-egui-web"}
MCP_PROFILES = {"cli", "desktop-egui", "mcp-stdio", "mcp-http"}
HTTP_PROFILES = {"service-axum", "web-leptos-fullstack"}
ROOT_MANIFEST = "Cargo.toml"
APP_MANIFEST = "crates/app/Cargo.toml"
RUNTIME_MANIFEST = "crates/starter-reference/Cargo.toml"


def plan(profile, preset, ordered, modules):
    """Choose default integrations using the capability set, including references."""
    selected = set(ordered)
    needed = set()
    if profile in DESKTOP and selected.intersection({"application-core", "desktop-shell", "documents"}):
        needed.add("reference-runtime")
    if profile == "desktop-egui" and not any(modules[i].get("provider") == "desktop-view" for i in ordered):
        if preset == "engineering" or {"canvas", "viewport"} <= selected:
            needed.add("engineering-demo")
        elif "data" in selected:
            needed.add("data-demo")
        elif "documents" in selected:
            needed.add("document-demo")
        elif "reference-runtime" in selected and "tasks" in selected:
            needed.add("desktop-product")
    if profile in {"mcp-http", "mcp-stdio"} and "application-core" in selected:
        needed.add("application-mcp")
    if profile in HTTP_PROFILES and "application-core" in selected:
        needed.add("application-http")
    return needed - selected


def entrypoints(profile, ordered, modules):
    selected = set(ordered)
    providers = [name for name in ordered if modules[name].get("provider") == "desktop-view"]
    if len(providers) > 1:
        raise IntegrationError(f"conflicting desktop workspaces: {', '.join(providers)}")
    if "reference-runtime" in selected:
        web = ["operations-web"] if profile == "desktop-egui-web" else []
        mcp = ["mcp-stdio", "mcp-live-http"] if "application-mcp" in selected else []
        return web + ["desktop", "operations-cli"] + mcp
    if "application-core" not in selected:
        return []
    if profile == "cli":
        return (["mcp-stdio"] if "application-mcp" in selected else []) + ["operations-cli"]
    if profile in HTTP_PROFILES:
        return ["application-http"]
    if profile in {"mcp-stdio", "mcp-http"}:
        return [profile]
    return [{
        "library": "operations-api",
        "mcp-client": "operations-cli-companion",
        "web-leptos-csr": "operations-web",
        "web-dioxus": "operations-web",
    }[profile]]


def guides(profile, interfaces):
    """Describe commands enabled by the exact composition."""
    if "operations-cli" in interfaces:
        command = "cargo run -p desktop" if profile in DESKTOP else "cargo run"
        yield f"`{command} -- --operations` lists the installed operation registry."
    if "operations-cli-companion" in interfaces:
        yield "`cargo run --bin operations -- --operations` lists the companion registry."
    if "mcp-stdio" in interfaces:
        command = {
            "cli": "cargo run -- --mcp",
            "desktop-egui": "cargo run -p desktop -- --mcp",
            "mcp-stdio": "cargo run",
        }[profile]
        yield f"`{command}` starts the stdio endpoint."
    if "mcp-live-http" in interfaces:
        yield "`cargo run -p desktop -- --mcp-gui` starts stdio with a shared window."
        yield (
            "`cargo run -p desktop -- --mcp-live` starts authenticated loopback HTTP. "
            "Set private `STARTER_SESSION_DIR` and a unique URL-safe "
            "`STARTER_MCP_TOKEN` of at least 32 characters. Run "
            "`cargo run -p desktop -- --sessions` to discover the instance, then match "
            "its ID against MCP capabilities before executing operations."
        )
    if "mcp-http" in interfaces:
        yield "`cargo run` starts the local Streamable HTTP server at `127.0.0.1:8000/mcp`."
    if "application-http" in interfaces:
        yield "Set `APP_OPERATIONS_TOKEN` before `cargo run` and authenticate requests to `/operations` and `/invoke`."


def cargo_dependency(crate, path, **extra):
    return {"dependencies": {crate: {"path": path, **extra}}}


class IntegrationError(ValueError):
    """The selected adapter cannot be connected to its base template."""


def replace_once(path, source, anchor, replacement):
    count = source.count(anchor)
    if count != 1 or anchor == replacement:
        raise IntegrationError(
            f"{path.name}: expected one integration anchor {anchor!r}, found {count}"
        )
    return source.replace(anchor, replacement, 1)


def edit(path, *patches):
    source = path.read_text(encoding="utf-8")
    for anchor, replacement in patches:
        source = replace_once(path, source, anchor, replacement)
    path.write_text(source, encoding="utf-8")


def install(project: Path, profile, ordered, modules, *, name, merge):
    changed, interfaces = {}, []
    selected = set(ordered)
    runtime = "reference-runtime" in selected
    if "application-mcp" in selected and profile not in MCP_PROFILES:
        raise ValueError(f"application-mcp has no entrypoint for {profile}")
    if "application-http" in selected and profile not in HTTP_PROFILES:
        raise ValueError(f"application-http has no entrypoint for {profile}")
    if runtime:
        runtime_features = []
        if "application-mcp" in selected:
            merge(changed.setdefault(RUNTIME_MANIFEST, {}),
                  cargo_dependency("starter-mcp", "../starter-mcp", optional=True))
            merge(changed[RUNTIME_MANIFEST], {"features": {"mcp": ["dep:starter-mcp"]}})
            runtime_features.append("mcp")
        if "desktop-product" in selected:
            merge(changed.setdefault(RUNTIME_MANIFEST, {}),
                  cargo_dependency("starter-tasks", "../starter-tasks", optional=True))
            merge(changed[RUNTIME_MANIFEST], {"features": {"tasks": ["dep:starter-tasks"]}})
        if profile == "desktop-egui-web" and "documents" in selected:
            merge(changed.setdefault(RUNTIME_MANIFEST, {}),
                  cargo_dependency("starter-notes", "../starter-notes", optional=True))
            merge(changed[RUNTIME_MANIFEST], {"features": {"notes": ["dep:starter-notes", "dep:web-sys"]}})
            runtime_features.append("notes")
        providers = [i for i in ordered if modules[i].get("provider") == "desktop-view"]
        if len(providers) > 1:
            raise ValueError(f"multiple desktop providers: {providers}")
        adapter = project / "crates/app/src/lib.rs"
        title = f'"{name}"'
        adapter_source = adapter.read_text(encoding="utf-8")
        for symbol in ("pub struct TemplateApp", "impl eframe::App for TemplateApp"):
            replace_once(adapter, adapter_source, symbol, "validated")
        desktop_entry = project / "crates/desktop/src/main.rs"
        replace_once(desktop_entry, desktop_entry.read_text(encoding="utf-8"),
                     "ui_app::TemplateApp::new(cc)", "validated")
        if "desktop-product" in selected:
            if providers:
                raise IntegrationError("general desktop view conflicts with a specialized workspace")
            for crate, path in (("starter-reference", "../starter-reference"), ("starter-core", "../starter-core"),
                                ("starter-desktop", "../starter-desktop")):
                flags = {"features": [*runtime_features, "tasks"]} if crate == "starter-reference" else {}
                merge(changed.setdefault(APP_MANIFEST, {}), cargo_dependency(crate, path, **flags))
            merge(changed[APP_MANIFEST], {
                "features": {"ui-inspection": ["starter-reference/ui-inspection"]},
            })
            adapter.write_text(
                "mod domain;\nmod view;\n"
                "pub fn run() -> starter_reference::Result<()> {\n"
                f"    starter_reference::launch({title}, domain::application(), |_, app| view::WorkspaceView::new(app))\n"
                "}\n", encoding="utf-8",
            )
        elif providers == ["lab-workbench"]:
            merge(changed.setdefault(APP_MANIFEST, {}), cargo_dependency("starter-lab", "../starter-lab"))
            merge(changed[APP_MANIFEST], {"features": {"ui-inspection": ["starter-lab/ui-inspection"]}})
            adapter.write_text(
                "pub fn run() -> starter_reference::Result<()> {\n"
                f"    starter_lab::run({title})\n"
                "}\n", encoding="utf-8",
            )
            merge(changed[APP_MANIFEST], cargo_dependency("starter-reference", "../starter-reference"))
            if runtime_features:
                merge(changed.setdefault("crates/starter-lab/Cargo.toml", {}),
                      cargo_dependency("starter-reference", "../starter-reference", features=runtime_features))
        elif providers:
            # One host and an application-provided domain view.
            merge(changed.setdefault(APP_MANIFEST, {}), cargo_dependency("starter-demo", "../starter-demo"))
            merge(changed[APP_MANIFEST], {"features": {"ui-inspection": ["starter-demo/ui-inspection"]}})
            adapter.write_text("pub use starter_demo::run;\n", encoding="utf-8")
            if runtime_features:
                merge(changed.setdefault("crates/starter-demo/Cargo.toml", {}),
                      cargo_dependency("starter-reference", "../starter-reference", features=runtime_features))
        else:
            extra = {"features": runtime_features} if runtime_features else {}
            merge(changed.setdefault(APP_MANIFEST, {}), cargo_dependency("starter-reference", "../starter-reference", **extra))
            merge(changed[APP_MANIFEST], {"features": {"ui-inspection": ["starter-reference/ui-inspection"]}})
            prefix = "notes" if "notes" in runtime_features else "standard"
            lines = [
                f'const TITLE: &str = {title};',
                '#[cfg(not(target_arch = "wasm32"))]',
                'pub fn run() -> starter_reference::Result<()> {',
                f'    starter_reference::{prefix}_launch(TITLE)',
                '}',
                'pub fn create(cc: &eframe::CreationContext<\'_>) -> Box<dyn eframe::App> {',
                f'    Box::new(starter_reference::{prefix}_window(cc, TITLE))',
                '}',
            ]
            adapter.write_text("\n".join(lines)+"\n", encoding="utf-8")
        desktop_entry.write_text(
            'fn main() -> std::process::ExitCode {\n'
            '    if let Err(error) = ui_app::run() {\n'
            '        eprintln!("{error}");\n'
            '        return std::process::ExitCode::FAILURE;\n'
            '    }\n'
            '    std::process::ExitCode::SUCCESS\n'
            '}\n', encoding="utf-8")
        merge(changed.setdefault("crates/desktop/Cargo.toml", {}), {"features": {"ui-inspection": ["ui_app/ui-inspection"]}})
        if profile == "desktop-egui-web":
            web = project / "crates/web/src/lib.rs"
            edit(web, ("Ok(Box::new(ui_app::TemplateApp::new(cc)))", "Ok(ui_app::create(cc))"),
                 ('.expect("web runner");', '.expect("web runner");\n        document.document_element().expect("html element")'
                  '.set_attribute("data-starter-ready", "true").expect("mark web runner ready");'))
            if "notes" in runtime_features:
                html = project / "crates/web/index.html"
                edit(html, ("</body>", '<output id="starter-note-status" role="status" '
                            'style="position:absolute;clip-path:inset(50%)"></output></body>'))
            interfaces.append("operations-web")
        interfaces.extend(["desktop", "operations-cli"])
        if "application-mcp" in selected:
            interfaces.extend(["mcp-stdio", "mcp-live-http"])

    if profile == "cli" and "application-core" in selected:
        path = project / "src/main.rs"
        body = '''    let mut operations = std::env::args().skip(1).peekable();
    if operations.peek().is_some_and(|arg| matches!(arg.as_str(), "--operations" | "--invoke")) {
        return starter_core::sample::cli(operations).map_err(anyhow::Error::msg);
    }
'''
        if "application-mcp" in selected:
            body += '''    if operations.peek().is_some_and(|arg| arg == "--mcp") {
        anyhow::ensure!(operations.len() == 1, "--mcp takes no arguments");
        let app = std::sync::Arc::new(std::sync::Mutex::new(starter_core::sample::application()));
        return starter_mcp::stdio(starter_mcp::Server::new(app, starter_core::sample::session()))
            .map_err(anyhow::Error::msg);
    }
'''
            merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-mcp", "crates/starter-mcp"))
            interfaces.append("mcp-stdio")
        edit(path, ("    let args = Cli::parse();", body + "    let args = Cli::parse();"))
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        interfaces.append("operations-cli")

    if profile in HTTP_PROFILES and "application-http" in selected:
        path = project / "src/main.rs"
        body = '''    let token = std::env::var("APP_OPERATIONS_TOKEN")?;
    let state = std::sync::Arc::new(std::sync::Mutex::new(starter_core::sample::application()));
    let operations = starter_http::router(state, starter_core::sample::session(), &token)
        .map_err(anyhow::Error::msg)?;
    axum::serve(listener, service_core::router().merge(operations))'''
        if profile == "service-axum":
            edit(path, ("    axum::serve(listener, service_core::router())", body))
        else:
            edit(path, (
                '    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;',
                '    let bind = std::env::var("APP_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());\n'
                '    let listener = tokio::net::TcpListener::bind(bind).await?;',
            ), ("    axum::serve(listener, router)", body.replace("service_core::router()", "router")))
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], cargo_dependency("starter-http", "crates/starter-http"))
        interfaces.append("application-http")

    if profile in {"mcp-stdio", "mcp-http"} and "application-mcp" in selected:
        path = project / "src/main.rs"
        code = path.read_text(encoding="utf-8")
        state = "let app = std::sync::Arc::new(std::sync::Mutex::new(starter_core::sample::application()));"
        server = "starter_mcp::Server::new(app, starter_core::sample::session())"
        if profile == "mcp-http":
            code = replace_once(path, code, "    let service = StreamableHttpService::new(", f"    {state}\n    let service = StreamableHttpService::new(")
            code = replace_once(path, code, "|| Ok(mcp_tools::EchoServer::new())", f"move || Ok({server.replace('new(app,','new(app.clone(),')})")
            code = replace_once(path, code, "    let tcp = tokio::net::TcpListener::bind(&addr).await?;", (
                "    let address: std::net::SocketAddr = addr.parse()?;\n"
                "    anyhow::ensure!(address.ip().is_loopback(), \"the application MCP profile binds only to loopback\");\n"
                "    let tcp = tokio::net::TcpListener::bind(address).await?;"
            ))
        else:
            code = replace_once(path, code, "    mcp_tools::EchoServer::new()", f"    {state}\n    {server}")
        path.write_text(code,encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], cargo_dependency("starter-mcp", "crates/starter-mcp"))
        interfaces.append(profile)

    if "application-core" in selected and profile in {"web-leptos-csr", "web-dioxus"}:
        path = project / "src/main.rs"
        text = path.read_text(encoding="utf-8")
        if profile == "web-leptos-csr":
            text = replace_once(path, text, "    let (count, set_count) = signal(0_i32);", """
    let app = RwSignal::new(starter_core::sample::application());
    let name = RwSignal::new("Workspace".to_owned());
    let draft = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
""".strip("\n"))
            text = replace_once(path, text, '''            <p>"Leptos CSR starter"</p>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                {move || count.get().to_string()}
            </button>
''', """            <h2>{move || name.get()}</h2>
            <label>"Workspace name"<input on:input=move |event| draft.set(event_target_value(&event)) /></label>
            <button on:click=move |_| {
                match starter_core::sample::invoke(&mut app.write(), "workspace.rename", serde_json::json!({"name": draft.get()})) {
                    Ok(response) => { name.set(response.result.as_str().unwrap_or_default().into()); error.set(String::new()); }
                    Err(response) => error.set(response.message),
                }
            }>"Rename workspace"</button>
            <p role="status">{move || error.get()}</p>
""")
        else:
            text = replace_once(path, text, "    let mut count = use_signal(|| 0_i32);", """
    let mut app = use_signal(starter_core::sample::application);
    let mut name = use_signal(|| "Workspace".to_owned());
    let mut draft = use_signal(String::new);
    let mut error = use_signal(String::new);
""".strip("\n"))
            text = replace_once(path, text, '''            p { "Dioxus web starter" }
            button { onclick: move |_| count += 1, "Count: {count}" }
''', """            h2 { "{name}" }
            label { "Workspace name" input { oninput: move |event| draft.set(event.value()) } }
            button {
                onclick: move |_| match starter_core::sample::invoke(
                    &mut app.write(), "workspace.rename", serde_json::json!({"name": draft()})
                ) {
                    Ok(response) => { name.set(response.result.as_str().unwrap_or_default().into()); error.set(String::new()); }
                    Err(response) => error.set(response.message),
                },
                "Rename workspace"
            }
            p { role: "status", "{error}" }
""")
        path.write_text(text,encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], {"dependencies": {"serde_json": "1"}})
        interfaces.append("operations-web")

    if profile == "mcp-client" and "application-core" in selected:
        import tomllib
        manifest = tomllib.loads((project / ROOT_MANIFEST).read_text(encoding="utf-8"))
        path = project / "src/bin/operations.rs"
        path.parent.mkdir(exist_ok=True)
        if path.exists() or path.is_symlink():
            raise IntegrationError("the operations companion would overwrite src/bin/operations.rs")
        path.write_text(
            "fn main() -> anyhow::Result<()> {\n"
            "    starter_core::sample::cli(std::env::args().skip(1)).map_err(anyhow::Error::msg)\n"
            "}\n",encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], {"package": {"default-run": manifest["package"]["name"]}})
        interfaces.append("operations-cli-companion")

    if "application-core" in selected and not interfaces and not runtime:
        candidate = project / "src/lib.rs"
        if profile == "library":
            source = candidate.read_text(encoding="utf-8")
            candidate.write_text(source + """
/// Operation registry and a portable settings example.
///
/// ```
/// use """ + name.replace("-", "_") + """::operations::sample;
/// let mut app = sample::application();
/// let reply = sample::invoke(&mut app, "workspace.rename",
///     serde_json::json!({"name": "Verified"})).unwrap();
/// assert_eq!(reply.result, "Verified");
/// ```
pub use starter_core as operations;
""", encoding="utf-8")
            merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
            merge(changed[ROOT_MANIFEST], {"dev-dependencies": {"serde_json": "1"}})
            interfaces.append("operations-api")
    if "application-core" in selected and not interfaces:
        raise IntegrationError(f"application-core has no entrypoint for {profile}")
    if interfaces != entrypoints(profile, ordered, modules):
        raise IntegrationError(f"entrypoint inventory differs from installed adapters: {interfaces}")
    return changed, interfaces
