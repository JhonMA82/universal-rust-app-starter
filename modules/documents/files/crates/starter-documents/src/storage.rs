use crate::{Envelope, Error, Format, History};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub type Checksum = [u8; 32];
pub struct SaveReport {
    pub checksum: Checksum,
    pub version: u32,
    pub revision: u64,
    pub bytes: usize,
    pub directory_synced: bool,
}

/// Native, typed document session with history and conflict-aware persistence.
///
/// The application supplies its format and chooses paths. Opening a document
/// starts a new in-memory undo history. Unknown envelope fields are preserved.
pub struct Document<T> {
    format: Format<T>,
    history: History<T>,
    binding: Option<(PathBuf, Checksum)>,
    extra: BTreeMap<String, Value>,
}

impl<T: Clone + DeserializeOwned + Serialize + PartialEq> Document<T> {
    pub fn new(format: Format<T>, initial: T, undo_capacity: usize) -> Result<Self, Error> {
        (format.validate)(&initial)?;
        Ok(Self {
            format,
            history: History::new(initial, undo_capacity),
            binding: None,
            extra: BTreeMap::new(),
        })
    }

    pub fn open(format: Format<T>, path: &Path, undo_capacity: usize) -> Result<Self, Error> {
        let path = absolute(path)?;
        let (document, checksum) = open(&path, &format)?;
        let mut history = History::new(document.content, undo_capacity);
        history.revision = document.revision;
        Ok(Self {
            format,
            history,
            binding: Some((path, checksum)),
            extra: document.extra,
        })
    }

    pub fn history(&self) -> &History<T> {
        &self.history
    }

    pub fn replace(&mut self, value: T) -> Result<bool, Error> {
        self.history.replace(value, self.format.validate)
    }

    pub fn undo(&mut self) -> Result<bool, Error> {
        self.history.undo()
    }

    pub fn redo(&mut self) -> Result<bool, Error> {
        self.history.redo()
    }

    /// Save to the current file or a new destination. An unrelated existing file
    /// is treated as a conflict. Failure leaves the save point and binding intact.
    pub fn save(&mut self, path: &Path) -> Result<SaveReport, Error> {
        let path = absolute(path)?;
        let expected = self
            .binding
            .as_ref()
            .filter(|(bound, _)| bound == &path)
            .map(|(_, checksum)| *checksum);
        let snapshot = self.history.snapshot();
        let content = Envelope {
            format: self.format.id.into(),
            version: self.format.version,
            revision: self.history.revision(),
            content: (*snapshot).clone(),
            extra: self.extra.clone(),
        };
        let report = save(&path, &self.format, &content, expected)?;
        self.binding = Some((path, report.checksum));
        self.history.mark_saved(snapshot);
        Ok(report)
    }
}

fn absolute(path: &Path) -> Result<PathBuf, Error> {
    let path = std::path::absolute(path)?;
    let filename = path
        .file_name()
        .ok_or_else(|| Error::Invalid("file name required".into()))?;
    let parent = path
        .parent()
        .ok_or_else(|| Error::Invalid("parent directory required".into()))?;
    Ok(parent.canonicalize()?.join(filename))
}

fn read(path: &Path, max: usize) -> Result<(Vec<u8>, Checksum), Error> {
    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(Error::Invalid("regular file required".into()));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take((max as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(Error::Limit);
    }
    let sum = Sha256::digest(&bytes).into();
    Ok((bytes, sum))
}
pub fn open<T: DeserializeOwned>(
    path: &Path,
    format: &Format<T>,
) -> Result<(Envelope<T>, Checksum), Error> {
    let (bytes, sum) = read(path, format.max_bytes)?;
    Ok((format.decode(&bytes)?, sum))
}
pub fn save<T: DeserializeOwned + Serialize + PartialEq>(
    path: &Path,
    format: &Format<T>,
    content: &Envelope<T>,
    expected: Option<Checksum>,
) -> Result<SaveReport, Error> {
    (format.validate)(&content.content)?;
    if content.format != format.id || content.version != format.version {
        return Err(Error::Format);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut lock_name = path
        .file_name()
        .ok_or_else(|| Error::Invalid("file name required".into()))?
        .to_owned();
    lock_name.push(".starter-lock");
    let lock = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join(lock_name))?;
    fs2::FileExt::lock_exclusive(&lock)?;
    let check = || {
        let actual = match read(path, format.max_bytes) {
            Ok((_, sum)) => Some(sum),
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if actual == expected {
            Ok(())
        } else {
            Err(Error::Conflict)
        }
    };
    check()?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    let mut bounded = Bounded {
        file: tmp.as_file_mut(),
        remaining: format.max_bytes,
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer_pretty(&mut bounded, content) {
        return if bounded.exceeded {
            Err(Error::Limit)
        } else {
            Err(error.into())
        };
    }
    let bytes = format.max_bytes - bounded.remaining;
    tmp.as_file().sync_all()?;
    let (decoded, checksum) = open(tmp.path(), format)?;
    if &decoded != content {
        return Err(Error::Invalid(
            "semantic round trip changed the document".into(),
        ));
    }
    check()?;
    if expected.is_some() {
        tmp.persist(path).map_err(|e| e.error)?;
    } else {
        tmp.persist_noclobber(path).map_err(|e| e.error)?;
    }
    Ok(SaveReport {
        checksum,
        version: content.version,
        revision: content.revision,
        bytes,
        directory_synced: cfg!(unix) && File::open(parent).and_then(|d| d.sync_all()).is_ok(),
    })
}
struct Bounded<'a> {
    file: &'a mut File,
    remaining: usize,
    exceeded: bool,
}
impl Write for Bounded<'_> {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        if input.len() > self.remaining {
            self.exceeded = true;
            return Err(std::io::Error::other("document size limit"));
        }
        let count = self.file.write(input)?;
        self.remaining -= count;
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_document_reopens_and_keeps_verified_save_point_on_conflict() {
        fn format() -> Format<String> {
            Format {
                id: "text",
                version: 1,
                max_bytes: 1024,
                migrate: |_, _| Err(Error::Format),
                validate: |text| {
                    if text.is_empty() {
                        Err(Error::Invalid("text required".into()))
                    } else {
                        Ok(())
                    }
                },
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("text.json");
        let mut document = Document::new(format(), "initial".into(), 4).unwrap();
        assert!(document.replace(String::new()).is_err());
        document.replace("edit".into()).unwrap();
        let revision = document.save(&path).unwrap().revision;
        assert!(!document.history().dirty());
        let mut reopened = Document::open(format(), &path, 4).unwrap();
        assert_eq!(reopened.history().value(), "edit");
        assert_eq!(reopened.history().revision(), revision);
        document.replace("another edit".into()).unwrap();
        document.save(&path).unwrap();
        reopened.replace("concurrent change".into()).unwrap();
        assert!(matches!(reopened.save(&path), Err(Error::Conflict)));
        assert!(reopened.history().dirty());
        reopened.undo().unwrap();
        assert!(!reopened.history().dirty());
        reopened.redo().unwrap();
        let copy = directory.path().join("copy.json");
        reopened.save(&copy).unwrap();
        assert!(!reopened.history().dirty());
        assert_eq!(
            Document::open(format(), &copy, 4)
                .unwrap()
                .history()
                .value(),
            "concurrent change"
        );
    }

    #[test]
    fn migration_save_and_conflict() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("note.json");
        let format = Format {
            id: "note",
            version: 2,
            max_bytes: 512,
            migrate: |_, value| {
                *value = serde_json::Value::String(value.to_string());
                Ok(())
            },
            validate: |_: &String| Ok(()),
        };
        std::fs::write(
            &path,
            r#"{"format":"note","version":1,"revision":2,"content":7,"future":true}"#,
        )
        .unwrap();
        let (doc, old) = open(&path, &format).unwrap();
        assert_eq!(doc.content, "7");
        assert_eq!(doc.extra["future"], true);
        let report = save(&path, &format, &doc, Some(old)).unwrap();
        assert_eq!(
            open(&path, &format).unwrap(),
            (doc.clone(), report.checksum)
        );
        assert!(matches!(
            save(&path, &format, &doc, Some(old)),
            Err(Error::Conflict)
        ));
        std::fs::write(&path, b"bad").unwrap();
        assert!(open(&path, &format).is_err());
    }
}
