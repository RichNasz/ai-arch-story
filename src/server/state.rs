use std::{
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::Serialize;

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct AppState {
    pub workspace_root: PathBuf,
}

impl AppState {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub fn diagrams_dir(&self) -> PathBuf {
        self.workspace_root.join("diagrams")
    }

    pub(super) fn diagram_paths(&self, name: &str) -> Result<DiagramPaths, InvalidDiagramName> {
        let name = DiagramName::parse(name)?;
        let directory = self.diagrams_dir().join(name.as_str());
        Ok(DiagramPaths { name, directory })
    }

    pub fn project_json_path(&self) -> PathBuf {
        self.workspace_root.join("project.json")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiagramName(String);

impl DiagramName {
    fn parse(value: &str) -> Result<Self, InvalidDiagramName> {
        let mut bytes = value.bytes();
        let Some(first) = bytes.next() else {
            return Err(InvalidDiagramName);
        };
        if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
            return Err(InvalidDiagramName);
        }

        let mut previous_was_hyphen = false;
        for byte in bytes {
            match byte {
                b'a'..=b'z' | b'0'..=b'9' => previous_was_hyphen = false,
                b'-' if !previous_was_hyphen => previous_was_hyphen = true,
                _ => return Err(InvalidDiagramName),
            }
        }

        if previous_was_hyphen {
            return Err(InvalidDiagramName);
        }
        Ok(Self(value.to_owned()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InvalidDiagramName;

#[derive(Clone, Debug)]
pub(super) struct DiagramPaths {
    name: DiagramName,
    directory: PathBuf,
}

impl DiagramPaths {
    pub(super) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(super) fn definition(&self) -> PathBuf {
        self.directory.join("diagram.json")
    }

    pub(super) fn output_directory(&self) -> PathBuf {
        self.directory.join("output")
    }

    pub(super) fn output(&self) -> PathBuf {
        self.output_directory()
            .join(format!("{}.html", self.name()))
    }
}

#[derive(Debug)]
pub(super) enum AtomicJsonWriteError {
    Serialize(serde_json::Error),
    Io(io::Error),
}

impl fmt::Display for AtomicJsonWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for AtomicJsonWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialize(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for AtomicJsonWriteError {
    fn from(error: serde_json::Error) -> Self {
        if error.is_io() {
            let kind = error.io_error_kind().unwrap_or(io::ErrorKind::Other);
            Self::Io(io::Error::new(kind, error))
        } else {
            Self::Serialize(error)
        }
    }
}

impl From<io::Error> for AtomicJsonWriteError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(super) fn atomic_write_json<T: Serialize + ?Sized>(
    destination: &Path,
    value: &T,
) -> Result<(), AtomicJsonWriteError> {
    let (temporary_path, mut temporary_file) = create_temporary_sibling(destination)?;

    let write_result = (|| -> Result<(), AtomicJsonWriteError> {
        serde_json::to_writer_pretty(&mut temporary_file, value)?;
        temporary_file.write_all(b"\n")?;
        temporary_file.flush()?;
        Ok(())
    })();

    if let Err(error) = write_result {
        drop(temporary_file);
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }

    drop(temporary_file);
    if let Err(error) = fs::rename(&temporary_path, destination) {
        let _ = fs::remove_file(&temporary_path);
        return Err(AtomicJsonWriteError::Io(error));
    }

    Ok(())
}

fn create_temporary_sibling(destination: &Path) -> io::Result<(PathBuf, File)> {
    let parent = destination.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "atomic JSON destination must have a parent directory",
        )
    })?;
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("json");

    loop {
        let nonce = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let temporary_path =
            parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        match options.open(&temporary_path) {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}
