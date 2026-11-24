#![forbid(unsafe_code)]
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Request {
    pub command_id: String,
    pub parameters: Value,
    pub session_id: String,
    pub document_id: Option<String>,
    pub request_id: Option<String>,
    pub expected_revision: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub revision: u64,
    pub result: Value,
    pub changed: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}
impl Error {
    pub fn new(code: &'static str, message: impl ToString) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}
pub struct Session {
    pub id: String,
    pub permissions: BTreeSet<String>,
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Command,
    Query,
    Session,
    Presentation,
    Document,
}
#[derive(Serialize)]
pub struct Operation {
    pub id: &'static str,
    pub description: &'static str,
    pub permission: &'static str,
    pub public: bool,
    pub kind: Kind,
    pub input_schema: Value,
    pub output_schema: Value,
}
pub struct Definition<S> {
    pub id: &'static str,
    pub description: &'static str,
    pub permission: &'static str,
    pub public: bool,
    pub available: fn(&S, &Request) -> bool,
}
type Query<S> = dyn Fn(&S, &Request) -> Result<Value, Error> + Send + Sync;
type Command<S> = dyn Fn(&mut S, &Request) -> Result<Value, Error> + Send + Sync;
enum Handler<S> {
    Query(Box<Query<S>>),
    Command(Box<Command<S>>),
}
struct Entry<S> {
    operation: Operation,
    available: fn(&S, &Request) -> bool,
    handler: Handler<S>,
}

#[derive(Serialize)]
pub struct Event {
    pub operation: String,
    pub revision: u64,
    pub document_id: Option<String>,
    pub session_id: String,
}
pub struct Application<S> {
    entries: BTreeMap<&'static str, Entry<S>>,
    state: S,
    revision: u64,
    retention: usize,
    receipts: VecDeque<(Request, Response)>,
    events: VecDeque<Event>,
}
impl<S: Clone + 'static> Application<S> {
    pub fn new(state: S, retention: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            state,
            revision: 0,
            retention,
            receipts: VecDeque::new(),
            events: VecDeque::new(),
        }
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn state(&self) -> &S {
        &self.state
    }
    pub fn events(&self) -> &VecDeque<Event> {
        &self.events
    }

    pub fn query<I: DeserializeOwned + JsonSchema, O: Serialize + JsonSchema>(
        &mut self,
        definition: Definition<S>,
        run: impl Fn(&S, I, &Request) -> Result<O, Error> + Send + Sync + 'static,
    ) -> Result<(), Error> {
        self.register::<I, O>(
            definition,
            Kind::Query,
            Handler::Query(Box::new(move |state, request| {
                encode(run(state, decode(&request.parameters)?, request)?)
            })),
        )
    }

    pub fn command<I: DeserializeOwned + JsonSchema, O: Serialize + JsonSchema>(
        &mut self,
        definition: Definition<S>,
        kind: Kind,
        run: impl Fn(&mut S, I, &Request) -> Result<O, Error> + Send + Sync + 'static,
    ) -> Result<(), Error> {
        if matches!(kind, Kind::Query) {
            return Err(Error::new("invalid_kind", "command has mutable state"));
        }
        self.register::<I, O>(
            definition,
            kind,
            Handler::Command(Box::new(move |state, request| {
                encode(run(state, decode(&request.parameters)?, request)?)
            })),
        )
    }
    fn register<I: JsonSchema, O: JsonSchema>(
        &mut self,
        definition: Definition<S>,
        kind: Kind,
        handler: Handler<S>,
    ) -> Result<(), Error> {
        let std::collections::btree_map::Entry::Vacant(slot) = self.entries.entry(definition.id)
        else {
            return Err(Error::new("duplicate_operation", definition.id));
        };
        slot.insert(Entry {
            operation: Operation {
                id: definition.id,
                description: definition.description,
                permission: definition.permission,
                public: definition.public,
                kind,
                input_schema: schema_for!(I).to_value(),
                output_schema: schema_for!(O).to_value(),
            },
            available: definition.available,
            handler,
        });
        Ok(())
    }
    pub fn discover<'a>(
        &'a self,
        session: &'a Session,
        public_only: bool,
    ) -> impl Iterator<Item = &'a Operation> {
        self.entries
            .values()
            .map(|entry| &entry.operation)
            .filter(move |op| {
                (!public_only || op.public) && session.permissions.contains(op.permission)
            })
    }
    /// Record an infrastructure acknowledgement, such as the saved version of a document.
    /// Domain changes must use a registered operation.
    pub fn acknowledge(&mut self, update: impl FnOnce(&mut S)) {
        update(&mut self.state);
    }

    pub fn invoke(
        &mut self,
        session: &Session,
        public_only: bool,
        request: Request,
    ) -> Result<Response, Error> {
        let entry = self
            .entries
            .get(request.command_id.as_str())
            .filter(|e| !public_only || e.operation.public)
            .ok_or_else(|| Error::new("unknown_operation", &request.command_id))?;
        if request.session_id != session.id
            || !session.permissions.contains(entry.operation.permission)
        {
            return Err(Error::new("forbidden", "session lacks permission"));
        }
        if let Some(id) = &request.request_id {
            if id.is_empty() || id.len() > 128 {
                return Err(Error::new(
                    "invalid_input",
                    "request ID must be 1–128 bytes",
                ));
            }
            if let Some((earlier, response)) = self.receipts.iter().find(|(earlier, _)| {
                earlier.session_id == request.session_id && earlier.request_id.as_ref() == Some(id)
            }) {
                return if earlier == &request {
                    Ok(response.clone())
                } else {
                    Err(Error::new("request_id_reused", "parameters differ"))
                };
            }
        }
        if request
            .expected_revision
            .is_some_and(|revision| revision != self.revision)
        {
            return Err(Error::new("conflict", "stale application revision"));
        }
        if !(entry.available)(&self.state, &request) {
            return Err(Error::new("unavailable", &request.command_id));
        }
        let (result, changed) = match &entry.handler {
            Handler::Query(run) => (run(&self.state, &request)?, false),
            Handler::Command(run) => {
                let revision = self
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| Error::new("internal", "revision exhausted"))?;
                let mut candidate = self.state.clone();
                let output = run(&mut candidate, &request)?;
                self.state = candidate;
                self.revision = revision;
                append(
                    &mut self.events,
                    self.retention,
                    Event {
                        operation: request.command_id.clone(),
                        revision,
                        document_id: request.document_id.clone(),
                        session_id: request.session_id.clone(),
                    },
                );
                (output, true)
            }
        };
        let response = Response {
            revision: self.revision,
            result,
            changed,
        };
        if request.request_id.is_some() {
            append(
                &mut self.receipts,
                self.retention,
                (request, response.clone()),
            );
        }
        Ok(response)
    }
}
fn append<T>(values: &mut VecDeque<T>, capacity: usize, value: T) {
    if capacity == 0 {
        return;
    }
    if values.len() == capacity {
        values.pop_front();
    }
    values.push_back(value);
}
fn decode<I: DeserializeOwned>(value: &Value) -> Result<I, Error> {
    I::deserialize(value).map_err(|error| Error::new("invalid_input", error))
}
fn encode<O: Serialize>(value: O) -> Result<Value, Error> {
    serde_json::to_value(value).map_err(|error| Error::new("internal", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn rejected_operation_leaves_no_state_or_revision_change() {
        let mut app = Application::new(0i32, 2);
        app.command(
            Definition {
                id: "add",
                description: "Add",
                permission: "write",
                public: true,
                available: |_, _| true,
            },
            Kind::Command,
            |state, by: i32, _| {
                *state += by;
                if by < 0 {
                    return Err(Error::new("domain", "negative"));
                }
                Ok(*state)
            },
        )
        .unwrap();
        let mut session = Session {
            id: "local".into(),
            permissions: BTreeSet::new(),
        };
        let mut req = Request {
            command_id: "add".into(),
            parameters: json!(-2),
            session_id: "local".into(),
            document_id: None,
            request_id: Some("one".into()),
            expected_revision: Some(0),
        };
        assert_eq!(
            app.invoke(&session, true, req.clone()).unwrap_err().code,
            "forbidden"
        );
        session.permissions.insert("write".into());
        assert_eq!(
            app.invoke(&session, true, req.clone()).unwrap_err().code,
            "domain"
        );
        assert_eq!((*app.state(), app.revision()), (0, 0));
        req.parameters = json!(5);
        let first = app.invoke(&session, true, req.clone()).unwrap();
        assert_eq!(app.invoke(&session, true, req.clone()).unwrap(), first);
        req.parameters = json!(6);
        assert_eq!(
            app.invoke(&session, true, req.clone()).unwrap_err().code,
            "request_id_reused"
        );
        req.request_id = None;
        assert_eq!(
            app.invoke(&session, true, req).unwrap_err().code,
            "conflict"
        );
        assert_eq!(app.events().len(), 1);
    }
}
