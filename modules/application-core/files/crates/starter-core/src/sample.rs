//! Small portable engine used by the generic composition roots.
use crate::{Application, Definition, Error, Kind, Request, Response, Session};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct Settings {
    name: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Rename {
    name: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}

fn definition(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
) -> Definition<Settings> {
    Definition {
        id,
        description,
        permission,
        public: true,
        available: |_, _| true,
    }
}

pub fn application() -> Application<Settings> {
    let mut app = Application::new(
        Settings {
            name: "Workspace".into(),
        },
        64,
    );
    app.query(
        definition("workspace.name", "Read the workspace name", "read"),
        |state, _: Empty, _| Ok(state.name.clone()),
    )
    .expect("unique operation");
    app.command(
        definition("workspace.rename", "Rename the workspace", "write"),
        Kind::Command,
        |state, rename: Rename, _| {
            let name = rename.name.trim();
            if name.is_empty() || name.len() > 128 {
                return Err(Error::new(
                    "invalid_name",
                    "workspace name must contain 1–128 bytes",
                ));
            }
            state.name = name.into();
            Ok(state.name.clone())
        },
    )
    .expect("unique operation");
    app
}

pub fn session() -> Session {
    Session {
        id: "local".into(),
        permissions: ["read".into(), "write".into()].into(),
    }
}

pub fn request(operation: &str, parameters: Value) -> Request {
    Request {
        command_id: operation.into(),
        parameters,
        session_id: session().id,
        request_id: None,
        document_id: None,
        expected_revision: None,
    }
}

pub fn invoke(
    app: &mut Application<Settings>,
    operation: &str,
    parameters: Value,
) -> Result<Response, Error> {
    app.invoke(&session(), true, request(operation, parameters))
}

pub fn capabilities(app: &Application<Settings>) -> Value {
    let session = session();
    json!({"session_id": session.id, "revision": app.revision(),
           "operations": app.discover(&session, true).collect::<Vec<_>>()})
}

pub fn cli(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let mut args = args.into_iter();
    let mut app = application();
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--operations"), None, None) => println!("{}", capabilities(&app)),
        (Some("--invoke"), Some(payload), None) => {
            let request = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
            match app.invoke(&session(), true, request) {
                Ok(value) => println!("{}", json!({"ok": value})),
                Err(error) => {
                    println!("{}", json!({"error": error}));
                    return Err(error.message);
                }
            }
        }
        _ => return Err("expected --operations or --invoke <request JSON>".into()),
    }
    Ok(())
}
