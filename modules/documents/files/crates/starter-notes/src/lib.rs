#![forbid(unsafe_code)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starter_core::{Application, Definition, Error, Kind, Request};
use starter_documents::{Format, History};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone)]
pub struct Domain {
    pub notes: BTreeMap<u64, History<String>>,
    next: u64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct New {
    pub text: Option<String>,
    pub recovered: Option<bool>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Edit {
    text: String,
    if_revision: Option<u64>,
}
#[derive(Deserialize, JsonSchema)]
struct Empty {}
#[derive(Serialize, JsonSchema)]
pub struct Note {
    pub id: u64,
    pub revision: u64,
    pub text: String,
    pub dirty: bool,
}
pub fn format() -> Format<String> {
    Format {
        id: "starter-note",
        version: 1,
        max_bytes: 512 * 1024,
        migrate: |_, _| Err(starter_documents::Error::Format),
        validate: |text| {
            if text.len() <= 64 * 1024 {
                Ok(())
            } else {
                Err(starter_documents::Error::Limit)
            }
        },
    }
}
fn error(e: impl ToString) -> Error {
    Error::new("document", e)
}
fn definition(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
) -> Definition<Domain> {
    Definition {
        id,
        description,
        permission,
        public: true,
        available: if matches!(id, "notes.list" | "notes.new") {
            |_, _| true
        } else {
            |state, request| {
                request
                    .document_id
                    .as_deref()
                    .and_then(|id| id.parse().ok())
                    .is_some_and(|id: u64| state.notes.contains_key(&id))
            }
        },
    }
}
fn id(req: &Request) -> Result<u64, Error> {
    req.document_id
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| error("numeric document ID required"))
}
pub fn note(state: &Domain, id: u64) -> Result<Note, Error> {
    let h = state
        .notes
        .get(&id)
        .ok_or_else(|| error("unknown document"))?;
    Ok(Note {
        id,
        revision: h.revision(),
        text: h.value().clone(),
        dirty: h.dirty(),
    })
}
pub fn application() -> Application<Domain> {
    let mut app = Application::new(
        Domain {
            notes: BTreeMap::from([(1, History::new(String::new(), 64))]),
            next: 2,
        },
        64,
    );
    app.query(
        definition("notes.list", "List open notes", "read"),
        |state, _: Empty, _| Ok(state.notes.keys().copied().collect::<Vec<_>>()),
    )
    .unwrap();
    app.query(
        definition("notes.get", "Read an open note", "read"),
        |state, _: Empty, req| note(state, id(req)?),
    )
    .unwrap();
    app.command(
        definition("notes.new", "Create an open note", "write"),
        Kind::Session,
        |state, new: New, _| {
            if state.notes.len() >= 64 {
                return Err(error("open note limit"));
            }
            let text = new.text.unwrap_or_default();
            (format().validate)(&text).map_err(error)?;
            let id = state.next;
            state.next = id
                .checked_add(1)
                .ok_or_else(|| error("note ID exhausted"))?;
            let mut history = History::new(text, 64);
            if new.recovered == Some(true) {
                history.mark_saved(Arc::new(String::new()));
            }
            state.notes.insert(id, history);
            Ok(id)
        },
    )
    .unwrap();
    app.command(
        definition("notes.edit", "Commit a note body", "write"),
        Kind::Document,
        |state, edit: Edit, req| {
            let id = id(req)?;
            let h = state
                .notes
                .get_mut(&id)
                .ok_or_else(|| error("unknown document"))?;
            if edit.if_revision.is_some_and(|v| v != h.revision()) {
                return Err(Error::new("conflict", "stale note revision"));
            }
            h.replace(edit.text, format().validate).map_err(error)?;
            note(state, id)
        },
    )
    .unwrap();
    for (operation, description, redo) in [
        ("notes.undo", "Undo note edit", false),
        ("notes.redo", "Redo note edit", true),
    ] {
        app.command(
            definition(operation, description, "write"),
            Kind::Document,
            move |state, _: Empty, req| {
                let id = id(req)?;
                let h = state
                    .notes
                    .get_mut(&id)
                    .ok_or_else(|| error("unknown document"))?;
                if redo { h.redo() } else { h.undo() }.map_err(error)?;
                note(state, id)
            },
        )
        .unwrap();
    }
    app.command(
        Definition {
            public: false,
            ..definition("notes.saved", "Record a verified write", "write")
        },
        Kind::Document,
        |state, saved: Edit, req| {
            let history = state
                .notes
                .get_mut(&id(req)?)
                .ok_or_else(|| error("unknown document"))?;
            let revision = saved
                .if_revision
                .ok_or_else(|| error("saved revision required"))?;
            (format().validate)(&saved.text).map_err(error)?;
            if revision > history.revision()
                || (revision == history.revision() && history.value() != &saved.text)
            {
                return Err(Error::new(
                    "conflict",
                    "saved snapshot does not match the document",
                ));
            }
            history.mark_saved(Arc::new(saved.text));
            Ok(())
        },
    )
    .unwrap();
    app
}
