#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid document: {0}")]
    Invalid(String),
    #[error("document changed externally")]
    Conflict,
    #[error("document exceeds size limit")]
    Limit,
    #[error("unsupported format or schema version")]
    Format,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
#[derive(Clone)]
pub struct History<T> {
    current: Arc<T>,
    saved: Arc<T>,
    undo: VecDeque<Arc<T>>,
    redo: Vec<Arc<T>>,
    revision: u64,
    capacity: usize,
}
impl<T: PartialEq> History<T> {
    pub fn new(value: T, capacity: usize) -> Self {
        let value = Arc::new(value);
        Self {
            saved: value.clone(),
            current: value,
            undo: VecDeque::new(),
            redo: Vec::new(),
            revision: 0,
            capacity,
        }
    }
    pub fn value(&self) -> &T {
        &self.current
    }
    pub fn snapshot(&self) -> Arc<T> {
        self.current.clone()
    }
    pub fn dirty(&self) -> bool {
        self.saved != self.current
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn mark_saved(&mut self, saved: Arc<T>) {
        self.saved = saved;
    }
    /// Replace the live value with a recovered draft. The saved point is kept,
    /// so the restored value reports dirty when it differs from the file.
    pub fn restore(&mut self, value: T, revision: u64) {
        self.current = Arc::new(value);
        self.undo.clear();
        self.redo.clear();
        self.revision = revision;
    }
    pub fn replace(
        &mut self,
        value: T,
        validate: impl FnOnce(&T) -> Result<(), Error>,
    ) -> Result<bool, Error> {
        validate(&value)?;
        if value == *self.current {
            return Ok(false);
        }
        self.advance()?;
        let previous = std::mem::replace(&mut self.current, Arc::new(value));
        self.remember(previous);
        self.redo.clear();
        Ok(true)
    }
    pub fn undo(&mut self) -> Result<bool, Error> {
        if self.undo.is_empty() {
            return Ok(false);
        }
        self.advance()?;
        let previous = self.undo.pop_back().expect("history exists");
        self.redo
            .push(std::mem::replace(&mut self.current, previous));
        Ok(true)
    }
    pub fn redo(&mut self) -> Result<bool, Error> {
        if self.redo.is_empty() {
            return Ok(false);
        }
        self.advance()?;
        let next = self.redo.pop().expect("future exists");
        let previous = std::mem::replace(&mut self.current, next);
        self.remember(previous);
        Ok(true)
    }
    fn remember(&mut self, value: Arc<T>) {
        if self.capacity == 0 {
            return;
        }
        if self.undo.len() == self.capacity {
            self.undo.pop_front();
        }
        self.undo.push_back(value);
    }
    fn advance(&mut self) -> Result<(), Error> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::Invalid("revision exhausted".into()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub format: String,
    pub version: u32,
    pub revision: u64,
    pub content: T,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
pub struct Format<T> {
    pub id: &'static str,
    pub version: u32,
    pub max_bytes: usize,
    pub migrate: fn(u32, &mut Value) -> Result<(), Error>,
    pub validate: fn(&T) -> Result<(), Error>,
}
impl<T> Clone for Format<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Format<T> {}
impl<T: DeserializeOwned> Format<T> {
    pub fn decode(&self, input: &[u8]) -> Result<Envelope<T>, Error> {
        if input.len() > self.max_bytes {
            return Err(Error::Limit);
        }
        let mut raw: Envelope<Value> = serde_json::from_slice(input)?;
        if raw.format != self.id || raw.version == 0 || raw.version > self.version {
            return Err(Error::Format);
        }
        while raw.version < self.version {
            (self.migrate)(raw.version, &mut raw.content)?;
            raw.version += 1;
        }
        let content = serde_json::from_value(raw.content)?;
        (self.validate)(&content)?;
        Ok(Envelope {
            format: raw.format,
            version: raw.version,
            revision: raw.revision,
            content,
            extra: raw.extra,
        })
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
