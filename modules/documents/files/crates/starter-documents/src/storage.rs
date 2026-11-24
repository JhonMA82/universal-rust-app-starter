use crate::{Envelope, Error, Format};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub type Checksum = [u8; 32];
pub struct SaveReport {
    pub checksum: Checksum,
    pub version: u32,
    pub revision: u64,
    pub bytes: usize,
    pub directory_synced: bool,
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
