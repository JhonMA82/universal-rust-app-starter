use crate::{Envelope, Error, Format, History};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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

const RECOVERY_METADATA: &str = "_starter_recovery";

/// Location for a dirty draft written by autosave.
///
/// Reuse the same directory and key across processes to find a draft after
/// restart. Use a stable key for an untitled draft or derive one from a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recovery {
    directory: PathBuf,
    key: String,
}
impl Recovery {
    pub fn new(directory: impl Into<PathBuf>, key: impl AsRef<str>) -> Self {
        let key: String = key
            .as_ref()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let key = match key.as_str() {
            "" | "." | ".." => "recovery".into(),
            _ => key,
        };
        Self {
            directory: directory.into(),
            key,
        }
    }

    /// Derive a stable key from the canonical parent and the document file name.
    pub fn for_path(directory: impl Into<PathBuf>, path: &Path) -> Result<Self, Error> {
        let path = StoredPath::new(&absolute(path)?);
        let hash = Sha256::digest(serde_json::to_vec(&path)?);
        Ok(Self::new(directory, format!("path-{hash:x}")))
    }

    /// Find readable drafts of the supplied format in a product-owned directory.
    /// The list is sorted by key. Foreign and malformed files are skipped.
    pub fn discover<T: DeserializeOwned>(
        directory: impl Into<PathBuf>,
        format: &Format<T>,
    ) -> Result<Vec<RecoveryDraft>, Error> {
        let directory = directory.into();
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut drafts = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(key) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
                continue;
            };
            let recovery = Self::new(&directory, key);
            if recovery.key() != key || !entry.file_type()?.is_file() {
                continue;
            }
            let Ok((document, _)) = open(&entry.path(), format) else {
                continue;
            };
            let Ok(base) = recovery_base(&document.extra) else {
                continue;
            };
            drafts.push(RecoveryDraft {
                recovery,
                original_path: base.map(|base| base.path.into_path()),
                revision: document.revision,
            });
        }
        drafts.sort_by(|left, right| left.recovery.key.cmp(&right.recovery.key));
        Ok(drafts)
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn path(&self) -> PathBuf {
        self.directory.join(format!("{}.json", self.key))
    }
    pub fn exists(&self) -> bool {
        self.path().is_file()
    }
}

/// A typed draft as discovered on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryDraft {
    pub recovery: Recovery,
    pub original_path: Option<PathBuf>,
    pub revision: u64,
}

#[derive(Clone)]
struct Binding {
    path: PathBuf,
    checksum: Checksum,
    revision: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct RecoveryBase {
    path: StoredPath,
    checksum: Checksum,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
enum StoredPath {
    Text(String),
    #[cfg(unix)]
    Unix(Vec<u8>),
    #[cfg(windows)]
    Windows(Vec<u16>),
}

impl StoredPath {
    fn new(path: &Path) -> Self {
        if let Some(text) = path.to_str() {
            return Self::Text(text.into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            Self::Unix(path.as_os_str().as_bytes().to_vec())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Self::Windows(path.as_os_str().encode_wide().collect())
        }
        #[cfg(not(any(unix, windows)))]
        Self::Text(path.to_string_lossy().into_owned())
    }

    fn into_path(self) -> PathBuf {
        match self {
            Self::Text(text) => text.into(),
            #[cfg(unix)]
            Self::Unix(bytes) => {
                use std::os::unix::ffi::OsStringExt;
                std::ffi::OsString::from_vec(bytes).into()
            }
            #[cfg(windows)]
            Self::Windows(units) => {
                use std::os::windows::ffi::OsStringExt;
                std::ffi::OsString::from_wide(&units).into()
            }
        }
    }
}

fn recovery_base(extra: &BTreeMap<String, Value>) -> Result<Option<RecoveryBase>, Error> {
    match extra.get(RECOVERY_METADATA) {
        Some(value) => Ok(Option::deserialize(value)?),
        None => Ok(None),
    }
}

/// Native, typed document session with history and conflict-aware persistence.
///
/// The application supplies its format and chooses paths. Opening a document
/// starts a new in-memory undo history. Unknown envelope fields are preserved.
pub struct Document<T> {
    format: Format<T>,
    history: History<T>,
    binding: Option<Binding>,
    extra: BTreeMap<String, Value>,
}
impl<T: Clone> Clone for Document<T> {
    fn clone(&self) -> Self {
        Self {
            format: self.format,
            history: self.history.clone(),
            binding: self.binding.clone(),
            extra: self.extra.clone(),
        }
    }
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
            binding: Some(Binding {
                path,
                checksum,
                revision: document.revision,
            }),
            extra: document.extra,
        })
    }

    pub fn history(&self) -> &History<T> {
        &self.history
    }

    /// The file associated with the last successful open or save.
    pub fn bound_path(&self) -> Option<&Path> {
        self.binding.as_ref().map(|binding| binding.path.as_path())
    }

    fn recovery_base(&self) -> Option<RecoveryBase> {
        self.binding.as_ref().map(|binding| RecoveryBase {
            path: StoredPath::new(&binding.path),
            checksum: binding.checksum,
        })
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
            .filter(|binding| binding.path == path)
            .map(|binding| binding.checksum);
        let snapshot = self.history.snapshot();
        let content = Envelope {
            format: self.format.id.into(),
            version: self.format.version,
            revision: self.history.revision(),
            content: (*snapshot).clone(),
            extra: self.extra.clone(),
        };
        let report = save(&path, &self.format, &content, expected)?;
        self.binding = Some(Binding {
            path,
            checksum: report.checksum,
            revision: report.revision,
        });
        self.history.mark_saved(snapshot);
        Ok(report)
    }

    /// Write the dirty draft to its recovery file. Autosave never moves the save
    /// point, so the document stays dirty until `save` succeeds.
    pub fn autosave(&self, recovery: &Recovery) -> Result<Option<SaveReport>, Error> {
        if !self.history.dirty() {
            return Ok(None);
        }
        std::fs::create_dir_all(recovery.directory())?;
        let path = recovery.path();
        let base = self.recovery_base();
        let mut extra = self.extra.clone();
        if extra
            .insert(RECOVERY_METADATA.into(), serde_json::to_value(&base)?)
            .is_some()
        {
            return Err(Error::Invalid("reserved recovery metadata field".into()));
        }
        let content = Envelope {
            format: self.format.id.into(),
            version: self.format.version,
            revision: self.history.revision(),
            content: (*self.history.snapshot()).clone(),
            extra,
        };
        let expected = match open(&path, &self.format) {
            Ok((previous, checksum)) => {
                let previous_base = recovery_base(&previous.extra)?;
                let superseded = self.binding.as_ref().is_some_and(|binding| {
                    previous.revision <= binding.revision
                        && previous_base
                            .as_ref()
                            .is_none_or(|base| base.path == StoredPath::new(&binding.path))
                });
                if !superseded && previous_base != base {
                    return Err(Error::Conflict);
                }
                if previous.revision >= content.revision {
                    return if previous == content {
                        Ok(None)
                    } else {
                        Err(Error::Conflict)
                    };
                }
                Some(checksum)
            }
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        save(&path, &self.format, &content, expected).map(Some)
    }

    /// Load a newer recovery draft into the live history. The save point is
    /// preserved, so the recovered content remains dirty.
    pub fn recover(&mut self, recovery: &Recovery) -> Result<bool, Error> {
        let (mut content, _) = match open(&recovery.path(), &self.format) {
            Ok(document) => document,
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error),
        };
        if content.revision <= self.history.revision() {
            return Ok(false);
        }
        if recovery_base(&content.extra)? != self.recovery_base() {
            return Err(Error::Conflict);
        }
        content.extra.remove(RECOVERY_METADATA);
        self.history.restore(content.content, content.revision);
        if !content.extra.is_empty() {
            self.extra = content.extra;
        }
        Ok(true)
    }

    pub fn discard_recovery(&self, recovery: &Recovery) -> Result<(), Error> {
        match std::fs::remove_file(recovery.path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
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

    fn text_format() -> Format<String> {
        Format {
            id: "text",
            version: 1,
            max_bytes: 4096,
            migrate: |_, _| Err(Error::Format),
            validate: |_| Ok(()),
        }
    }

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

    #[test]
    fn autosave_and_recovery_preserve_the_save_point() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("document.json");
        let recovery = Recovery::new(directory.path(), "run:1");
        assert_eq!(recovery.key(), "run-1");
        let mut document = Document::new(text_format(), "saved".into(), 4).unwrap();
        assert!(document.autosave(&recovery).unwrap().is_none());
        document.replace("draft".into()).unwrap();
        assert_eq!(document.autosave(&recovery).unwrap().unwrap().revision, 1);
        assert!(document.history().dirty(), "autosave must not mark saved");
        document.save(&path).unwrap();
        document.replace("newer".into()).unwrap();
        document.autosave(&recovery).unwrap().unwrap();
        let mut reopened = Document::open(text_format(), &path, 4).unwrap();
        assert_eq!(reopened.history().value(), "draft");
        assert!(reopened.recover(&recovery).unwrap());
        assert_eq!(reopened.history().value(), "newer");
        assert!(reopened.history().dirty());
        assert!(
            !reopened.recover(&recovery).unwrap(),
            "same revision is stale"
        );
        reopened.discard_recovery(&recovery).unwrap();
        assert!(!recovery.exists());
        assert!(!reopened.recover(&recovery).unwrap());
    }

    #[test]
    fn recovery_identity_and_discovery_survive_process_restart() {
        const TEST_DIR: &str = "STARTER_DOCUMENT_RECOVERY_PROCESS_DIRECTORY";
        const TEST_PHASE: &str = "STARTER_DOCUMENT_RECOVERY_PROCESS_PHASE";

        if let Some(directory) = std::env::var_os(TEST_DIR).map(PathBuf::from) {
            let recovery_dir = directory.join("recovery");
            let path = directory.join("report.json");
            let other_path = directory.join("second report.json");
            let format = text_format();
            let untitled = Recovery::new(&recovery_dir, "untitled");
            match std::env::var(TEST_PHASE).unwrap().as_str() {
                "write" => {
                    for (path, draft) in [(&path, "first draft"), (&other_path, "second draft")] {
                        let mut document = Document::new(format, "saved".into(), 4).unwrap();
                        document.save(path).unwrap();
                        let recovery = Recovery::for_path(&recovery_dir, path).unwrap();
                        document.replace(draft.into()).unwrap();
                        assert!(document.autosave(&recovery).unwrap().is_some());
                    }
                    let mut document = Document::new(format, "blank".into(), 4).unwrap();
                    document.replace("untitled draft".into()).unwrap();
                    assert!(document.autosave(&untitled).unwrap().is_some());
                    std::fs::write(recovery_dir.join("corrupt.json"), b"broken").unwrap();
                }
                "read" => {
                    let drafts = Recovery::discover(&recovery_dir, &format).unwrap();
                    assert_eq!(drafts.len(), 3);
                    assert!(
                        drafts
                            .windows(2)
                            .all(|pair| pair[0].recovery.key() < pair[1].recovery.key())
                    );
                    let mut document = Document::open(format, &path, 4).unwrap();
                    let bound_path = document.bound_path().unwrap();
                    let from_file =
                        Recovery::for_path(&recovery_dir, &directory.join(".").join("report.json"))
                            .unwrap();
                    let found = drafts
                        .iter()
                        .find(|draft| draft.original_path.as_deref() == Some(bound_path))
                        .unwrap();
                    assert_eq!(found.recovery, from_file);
                    assert_eq!(found.revision, 1);
                    assert_eq!(document.history().value(), "saved");

                    let other = Recovery::for_path(&recovery_dir, &other_path).unwrap();
                    assert!(matches!(document.recover(&untitled), Err(Error::Conflict)));
                    assert!(matches!(document.recover(&other), Err(Error::Conflict)));
                    assert!(document.recover(&found.recovery).unwrap());
                    assert_eq!(document.history().value(), "first draft");
                    assert!(document.history().dirty());
                    assert!(!document.recover(&found.recovery).unwrap());
                    assert_eq!(open(&path, &format).unwrap().0.content, "saved");
                    document.discard_recovery(&found.recovery).unwrap();
                    assert!(!found.recovery.exists());
                    assert!(other.exists());

                    assert!(drafts.iter().any(|draft| {
                        draft.original_path.is_none() && draft.recovery == untitled
                    }));
                    let mut blank = Document::new(format, "blank".into(), 4).unwrap();
                    assert!(blank.recover(&untitled).unwrap());
                    assert_eq!(blank.history().value(), "untitled draft");
                    assert!(blank.history().dirty());
                }
                phase => panic!("unexpected recovery test phase {phase}"),
            }
            return;
        }

        let directory = tempfile::tempdir().unwrap();
        let executable = std::env::current_exe().unwrap();
        for phase in ["write", "read"] {
            let output = std::process::Command::new(&executable)
                .args([
                    "--exact",
                    "storage::tests::recovery_identity_and_discovery_survive_process_restart",
                    "--nocapture",
                ])
                .env(TEST_DIR, directory.path())
                .env(TEST_PHASE, phase)
                .output()
                .unwrap();
            assert!(
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
                "{phase} subprocess failed: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }

    #[test]
    fn recovery_protects_newer_drafts_and_rejects_a_changed_save_base() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("report.json");
        let recovery_dir = directory.path().join("recovery");
        let recovery = Recovery::for_path(recovery_dir, &path).unwrap();
        let format = text_format();
        let mut original = Document::new(format, "saved".into(), 4).unwrap();
        original.save(&path).unwrap();
        let mut competing = Document::open(format, &path, 4).unwrap();

        original.replace("first change".into()).unwrap();
        original.replace("newest draft".into()).unwrap();
        assert!(original.autosave(&recovery).unwrap().is_some());
        assert!(original.autosave(&recovery).unwrap().is_none());
        competing.replace("older draft".into()).unwrap();
        assert!(matches!(
            competing.autosave(&recovery),
            Err(Error::Conflict)
        ));
        competing.replace("other draft".into()).unwrap();
        assert!(matches!(
            competing.autosave(&recovery),
            Err(Error::Conflict)
        ));

        let (mut saved, checksum) = open(&path, &format).unwrap();
        saved.content = "changed on disk".into();
        save(&path, &format, &saved, Some(checksum)).unwrap();
        let mut opened = Document::open(format, &path, 4).unwrap();
        assert!(matches!(opened.recover(&recovery), Err(Error::Conflict)));
        assert_eq!(opened.history().value(), "changed on disk");
        assert!(!opened.history().dirty());
        assert!(recovery.exists());
    }
}
