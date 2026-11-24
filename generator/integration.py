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
    if profile in {"mcp-http", "mcp-stdio"} and "application-core" in selected:
        needed.add("application-mcp")
    if profile in HTTP_PROFILES and "application-core" in selected:
        needed.add("application-http")
    return needed - selected


def cargo_dependency(crate, path, **extra):
    return {"dependencies": {crate: {"path": path, **extra}}}


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
        if profile == "desktop-egui-web" and "documents" in selected:
            merge(changed.setdefault(RUNTIME_MANIFEST, {}),
                  cargo_dependency("starter-notes", "../starter-notes", optional=True))
            merge(changed[RUNTIME_MANIFEST], {"features": {"notes": ["dep:starter-notes"]}})
            runtime_features.append("notes")
        providers = [i for i in ordered if modules[i].get("provider") == "desktop-view"]
        if len(providers) > 1:
            raise ValueError(f"multiple desktop providers: {providers}")
        adapter = project / "crates/app/src/lib.rs"
        title = f'"{name}"'
        if providers:
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
        (project / "crates/desktop/src/main.rs").write_text(
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
            web.write_text(web.read_text(encoding="utf-8").replace("Ok(Box::new(ui_app::TemplateApp::new(cc)))", "Ok(ui_app::create(cc))").replace(
                '.expect("web runner");', '.expect("web runner");\n        document.document_element().expect("html element")'
                '.set_attribute("data-starter-ready", "true").expect("mark web runner ready");'), encoding="utf-8")
        interfaces.extend(["desktop", "operations-cli"])
        if "application-mcp" in selected:
            interfaces.append("mcp-stdio")

    if profile == "cli" and "application-core" in selected:
        path = project / "src/main.rs"
        text = path.read_text(encoding="utf-8")
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
        path.write_text(text.replace("    let args = Cli::parse();",body+"    let args = Cli::parse();"),encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        interfaces.append("operations-cli")

    if profile in HTTP_PROFILES and "application-http" in selected:
        path = project / "src/main.rs"
        text = path.read_text(encoding="utf-8")
        body = '''    let token = std::env::var("APP_OPERATIONS_TOKEN")?;
    let state = std::sync::Arc::new(std::sync::Mutex::new(starter_core::sample::application()));
    let operations = starter_http::router(state, starter_core::sample::session(), &token)
        .map_err(anyhow::Error::msg)?;
    axum::serve(listener, service_core::router().merge(operations))'''
        if profile == "service-axum":
            code = text.replace("    axum::serve(listener, service_core::router())", body)
        else:
            code = text.replace("    axum::serve(listener, router)", body.replace("service_core::router()", "router"))
        path.write_text(code,encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], cargo_dependency("starter-http", "crates/starter-http"))
        interfaces.append("application-http")

    if profile in {"mcp-stdio", "mcp-http"} and "application-mcp" in selected:
        path = project / "src/main.rs"
        code = path.read_text(encoding="utf-8")
        state = "let app = std::sync::Arc::new(std::sync::Mutex::new(starter_core::sample::application()));"
        server = "starter_mcp::Server::new(app, starter_core::sample::session())"
        if profile == "mcp-http":
            code = code.replace("    let service = StreamableHttpService::new(", f"    {state}\n    let service = StreamableHttpService::new(")
            code = code.replace("|| Ok(mcp_tools::EchoServer::new())", f"move || Ok({server.replace('new(app,','new(app.clone(),')})")
            code = code.replace("    let tcp = tokio::net::TcpListener::bind(&addr).await?;", (
                "    let address: std::net::SocketAddr = addr.parse()?;\n"
                "    anyhow::ensure!(address.ip().is_loopback(), \"the application MCP profile binds only to loopback\");\n"
                "    let tcp = tokio::net::TcpListener::bind(address).await?;"
            ))
        else:
            code = code.replace("    mcp_tools::EchoServer::new()", f"    {state}\n    {server}")
        path.write_text(code,encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], cargo_dependency("starter-mcp", "crates/starter-mcp"))
        interfaces.append(profile)

    if "application-core" in selected and profile in {"web-leptos-csr", "web-dioxus"}:
        path = project / "src/main.rs"
        text = path.read_text(encoding="utf-8")
        if profile == "web-leptos-csr":
            text = text.replace("    let (count, set_count) = signal(0_i32);", """
    let app = RwSignal::new(starter_core::sample::application());
    let name = RwSignal::new("Workspace".to_owned());
    let draft = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
""".strip("\n"))
            start = text.index('            <p>')
            end = text.index('        </main>',start)
            text = text[:start] + """            <h2>{move || name.get()}</h2>
            <label>"Workspace name"<input on:input=move |event| draft.set(event_target_value(&event)) /></label>
            <button on:click=move |_| {
                match starter_core::sample::invoke(&mut app.write(), "workspace.rename", serde_json::json!({"name": draft.get()})) {
                    Ok(response) => { name.set(response.result.as_str().unwrap_or_default().into()); error.set(String::new()); }
                    Err(response) => error.set(response.message),
                }
            }>"Rename workspace"</button>
            <p role="status">{move || error.get()}</p>
""" + text[end:]
        else:
            text = text.replace("    let mut count = use_signal(|| 0_i32);", """
    let mut app = use_signal(starter_core::sample::application);
    let mut name = use_signal(|| "Workspace".to_owned());
    let mut draft = use_signal(String::new);
    let mut error = use_signal(String::new);
""".strip("\n"))
            start = text.index('            p { "Dioxus web starter" }')
            end = text.index('        }',start)
            text = text[:start] + """            h2 { "{name}" }
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
""" + text[end:]
        path.write_text(text,encoding="utf-8")
        merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
        merge(changed[ROOT_MANIFEST], {"dependencies": {"serde_json": "1"}})
        interfaces.append("operations-web")

    if profile == "mcp-client" and "application-core" in selected:
        import tomllib
        manifest = tomllib.loads((project / ROOT_MANIFEST).read_text(encoding="utf-8"))
        path = project / "src/bin/operations.rs"
        path.parent.mkdir(exist_ok=True)
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
            candidate.write_text(candidate.read_text(encoding="utf-8") + "\n/// Operation registry and a portable settings example.\npub use starter_core as operations;\n", encoding="utf-8")
            merge(changed.setdefault(ROOT_MANIFEST, {}), cargo_dependency("starter-core", "crates/starter-core"))
            interfaces.append("operations-api")
    return changed, interfaces
