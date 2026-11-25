use schemars::JsonSchema;
use serde::Deserialize;
use starter_core::{Application, Definition, Error, Kind};

#[derive(Clone)]
pub struct Workspace {
    name: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Rename {
    name: String,
}

pub fn application() -> Application<Workspace> {
    let mut app = Application::new(
        Workspace {
            name: "Workspace".into(),
        },
        64,
    );
    app.query(
        operation("workspace.name", "Read workspace name", "read"),
        |state, _: Empty, _| Ok(state.name.clone()),
    )
    .expect("unique operation");
    app.command(
        operation("workspace.rename", "Rename workspace", "write"),
        Kind::Command,
        |state, input: Rename, _| {
            let name = input.name.trim();
            if name.is_empty() || name.len() > 128 {
                return Err(Error::new("invalid_name", "Use a name with 1–128 bytes"));
            }
            state.name = name.into();
            Ok(state.name.clone())
        },
    )
    .expect("unique operation");
    app
}

fn operation(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
) -> Definition<Workspace> {
    Definition {
        id,
        description,
        permission,
        public: true,
        available: |_, _| true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use starter_core::Request;

    #[test]
    fn workspace_rename_validates_and_commits_through_the_registry() {
        let mut app = application();
        let session = starter_reference::local_session();
        let mut request = Request {
            command_id: "workspace.rename".into(),
            parameters: json!({"name": " "}),
            session_id: session.id.clone(),
            document_id: None,
            request_id: None,
            expected_revision: Some(0),
        };
        assert!(app.invoke(&session, true, request.clone()).is_err());
        assert_eq!(app.revision(), 0);
        request.parameters = json!({"name": " My app "});
        let result = app.invoke(&session, true, request).unwrap();
        assert_eq!(result.result, "My app");
        assert_eq!(result.revision, 1);
    }
}
