#![forbid(unsafe_code)]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starter_core::{Application, Definition, Error, Kind};
use starter_documents::{Format, storage::Document};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionCheck {
    pub label: String,
    pub passed: bool,
    #[serde(default)]
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionReport {
    pub reference: String,
    pub site: String,
    pub checks: Vec<InspectionCheck>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Snapshot {
    pub reference: String,
    pub site: String,
    pub passed: usize,
    pub total: usize,
    pub revision: u64,
    pub dirty: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Edit {
    report: InspectionReport,
    if_revision: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Empty {}

/// The product-owned format. A new product registers one of these and reuses
/// the storage session instead of reimplementing envelopes and checksums.
pub fn format() -> Format<InspectionReport> {
    Format {
        id: "starter-inspection",
        version: 1,
        max_bytes: 64 * 1024,
        migrate: |_, _| Err(starter_documents::Error::Format),
        validate: |report| {
            let reference = report.reference.trim();
            let site = report.site.trim();
            if reference.is_empty() || reference.len() > 64 {
                return Err(starter_documents::Error::Invalid(
                    "reference must contain 1–64 bytes".into(),
                ));
            }
            if site.is_empty() || site.len() > 128 {
                return Err(starter_documents::Error::Invalid(
                    "site must contain 1–128 bytes".into(),
                ));
            }
            if report.checks.len() > 64 {
                return Err(starter_documents::Error::Limit);
            }
            for check in &report.checks {
                if check.label.trim().is_empty()
                    || check.label.len() > 96
                    || check.note.len() > 256
                {
                    return Err(starter_documents::Error::Invalid(
                        "checks need a 1–96 byte label and a note of at most 256 bytes".into(),
                    ));
                }
            }
            Ok(())
        },
    }
}

fn definition(
    id: &'static str,
    description: &'static str,
    permission: &'static str,
) -> Definition<Document<InspectionReport>> {
    Definition {
        id,
        description,
        permission,
        public: true,
        available: |_, _| true,
    }
}

fn document_error(error: starter_documents::Error) -> Error {
    Error::new("document", error)
}

fn snapshot(document: &Document<InspectionReport>) -> Snapshot {
    let report = document.history().value();
    Snapshot {
        reference: report.reference.clone(),
        site: report.site.clone(),
        passed: report.checks.iter().filter(|check| check.passed).count(),
        total: report.checks.len(),
        revision: document.history().revision(),
        dirty: document.history().dirty(),
    }
}

pub fn application() -> Application<Document<InspectionReport>> {
    let initial = InspectionReport {
        reference: "INS-001".into(),
        site: "Unassigned site".into(),
        checks: vec![InspectionCheck {
            label: "Confirm site access".into(),
            passed: false,
            note: String::new(),
        }],
    };
    let document = Document::new(format(), initial, 128).expect("valid initial report");
    let mut app = Application::new(document, 128);
    app.query(
        definition("inspection.get", "Read the inspection report", "read"),
        |document, _: Empty, _| Ok(snapshot(document)),
    )
    .unwrap();
    app.command(
        definition("inspection.edit", "Replace the inspection report", "write"),
        Kind::Document,
        |document, edit: Edit, _| {
            if edit.if_revision.is_some_and(|revision| revision != document.history().revision()) {
                return Err(Error::new("conflict", "stale inspection revision"));
            }
            document.replace(edit.report).map_err(document_error)?;
            Ok(snapshot(document))
        },
    )
    .unwrap();
    for (id, description, redo) in [
        ("inspection.undo", "Undo an inspection edit", false),
        ("inspection.redo", "Redo an inspection edit", true),
    ] {
        app.command(
            definition(id, description, "write"),
            Kind::Document,
            move |document, _: Empty, _| {
                if redo {
                    document.redo()
                } else {
                    document.undo()
                }
                .map_err(document_error)?;
                Ok(snapshot(document))
            },
        )
        .unwrap();
    }
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use starter_core::{Request, Session};

    fn session() -> Session {
        Session {
            id: "test".into(),
            permissions: ["read".into(), "write".into()].into(),
        }
    }

    fn request(command_id: &str, parameters: serde_json::Value) -> Request {
        Request {
            command_id: command_id.into(),
            parameters,
            session_id: "test".into(),
            document_id: None,
            request_id: None,
            expected_revision: None,
        }
    }

    #[test]
    fn registered_format_edits_undoes_and_reports_dirty() {
        let mut app = application();
        let report = json!({
            "reference": "INS-9",
            "site": "Bridge 7",
            "checks": [
                {"label": "Guardrail", "passed": true, "note": "Verified"},
                {"label": "Signage", "passed": false, "note": ""}
            ]
        });
        let response = app
            .invoke(&session(), false, request("inspection.edit", json!({"report": report})))
            .unwrap();
        let edited: Snapshot = serde_json::from_value(response.result).unwrap();
        assert_eq!(
            (edited.reference.as_str(), edited.site.as_str(), edited.passed, edited.total, edited.revision, edited.dirty),
            ("INS-9", "Bridge 7", 1, 2, 1, true)
        );
        let undone = app
            .invoke(&session(), false, request("inspection.undo", json!({})))
            .unwrap();
        let undone: Snapshot = serde_json::from_value(undone.result).unwrap();
        assert_eq!((undone.site.as_str(), undone.revision), ("Unassigned site", 2));
        let stale = app.invoke(
            &session(),
            false,
            request(
                "inspection.edit",
                json!({"report": {"reference": "INS-9", "site": "Bridge 7", "checks": []}, "if_revision": 0}),
            ),
        );
        assert_eq!(stale.unwrap_err().code, "conflict");
    }

    #[test]
    fn invalid_reports_are_rejected_without_a_revision() {
        let mut app = application();
        let invalid = json!({"reference": " ", "site": "Bridge 7", "checks": []});
        let response = app.invoke(
            &session(),
            false,
            request("inspection.edit", json!({"report": invalid})),
        );
        assert_eq!(response.unwrap_err().code, "document");
        let current = app
            .invoke(&session(), false, request("inspection.get", json!({})))
            .unwrap();
        let current: Snapshot = serde_json::from_value(current.result).unwrap();
        assert_eq!((current.revision, current.dirty), (0, false));
    }
}
