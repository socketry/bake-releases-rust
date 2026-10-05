// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Reusable release-document tasks, inspired by Samuel Williams's Ruby
//! `bake-releases`: <https://github.com/ioquatix/bake-releases> (MIT).
//! This implementation preserves Markdown bytes rather than re-rendering them.
mod document;

pub use document::{extract_notes, update_document};

use bake::{Context, Error, Result};
use std::fs;
use std::fs::{File, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn persist_file(temporary: tempfile::NamedTempFile, path: &Path) -> Result<()> {
    temporary
        .persist(path)
        .map(|_| ())
        .map_err(|error| Error::from(error.error))
}

trait UpdateIo {
    fn create_temporary(&mut self, directory: &Path) -> io::Result<tempfile::NamedTempFile>;
    fn write(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()>;
    fn permissions(&mut self, path: &Path) -> io::Result<Permissions>;
    fn set_permissions(&mut self, file: &File, permissions: Permissions) -> io::Result<()>;
    fn sync_all(&mut self, file: &File) -> io::Result<()>;
    fn persist(&mut self, temporary: tempfile::NamedTempFile, path: &Path) -> Result<()>;
}

struct SystemUpdateIo;

impl UpdateIo for SystemUpdateIo {
    fn create_temporary(&mut self, directory: &Path) -> io::Result<tempfile::NamedTempFile> {
        tempfile::NamedTempFile::new_in(directory)
    }

    fn write(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
        file.write_all(contents)
    }

    fn permissions(&mut self, path: &Path) -> io::Result<Permissions> {
        fs::metadata(path).map(|metadata| metadata.permissions())
    }

    fn set_permissions(&mut self, file: &File, permissions: Permissions) -> io::Result<()> {
        file.set_permissions(permissions)
    }

    fn sync_all(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }

    fn persist(&mut self, temporary: tempfile::NamedTempFile, path: &Path) -> Result<()> {
        persist_file(temporary, path)
    }
}

fn update_file_with(
    directory: &Path,
    path: &Path,
    updated: &str,
    operations: &mut impl UpdateIo,
) -> Result<()> {
    let mut temporary = operations.create_temporary(directory)?;
    operations.write(temporary.as_file_mut(), updated.as_bytes())?;
    let permissions = operations.permissions(path)?;
    operations.set_permissions(temporary.as_file(), permissions)?;
    operations.sync_all(temporary.as_file())?;
    operations.persist(temporary, path)?;
    Ok(())
}

/// Extract the Markdown body beneath an exact release heading.
#[bake::task]
pub fn notes(
    context: &mut Context,
    version: String,
    #[bake(
        default = "releases.md",
        help = "Release document relative to the project root."
    )]
    path: PathBuf,
) -> Result<String> {
    let path = context.root().join(path);
    let document = fs::read_to_string(&path)
        .map_err(|error| Error::new(format!("{}: {error}", path.display())))?;
    Ok(extract_notes(&document, &version)?.to_owned())
}

/// Rename Unreleased to a release version, preserving the rest of the document.
#[bake::task]
pub fn update(
    context: &mut Context,
    version: String,
    #[bake(default = "releases.md")] path: PathBuf,
) -> Result<()> {
    let path = context.root().join(path).canonicalize()?;
    let document = fs::read_to_string(&path)?;
    let updated = update_document(&document, &version)?;
    let directory = path.parent().unwrap_or(path.as_path());
    update_file_with(directory, &path, &updated, &mut SystemUpdateIo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_file_reports_a_missing_destination_directory() {
        let directory = tempfile::tempdir().unwrap();
        let temporary = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
        let destination = directory.path().join("missing/releases.md");

        assert!(persist_file(temporary, &destination).is_err());
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum UpdateOperation {
        CreateTemporary,
        Write,
        Permissions,
        SetPermissions,
        Sync,
        Persist,
    }

    struct FailingUpdateIo {
        fail_at: Option<UpdateOperation>,
        system: SystemUpdateIo,
    }

    impl FailingUpdateIo {
        fn new(fail_at: Option<UpdateOperation>) -> Self {
            Self {
                fail_at,
                system: SystemUpdateIo,
            }
        }

        fn fail_if(&self, operation: UpdateOperation) -> io::Result<()> {
            if self.fail_at == Some(operation) {
                Err(io::Error::other("simulated update failure"))
            } else {
                Ok(())
            }
        }
    }

    impl UpdateIo for FailingUpdateIo {
        fn create_temporary(&mut self, directory: &Path) -> io::Result<tempfile::NamedTempFile> {
            self.fail_if(UpdateOperation::CreateTemporary)?;
            self.system.create_temporary(directory)
        }

        fn write(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
            self.fail_if(UpdateOperation::Write)?;
            self.system.write(file, contents)
        }

        fn permissions(&mut self, path: &Path) -> io::Result<Permissions> {
            self.fail_if(UpdateOperation::Permissions)?;
            self.system.permissions(path)
        }

        fn set_permissions(&mut self, file: &File, permissions: Permissions) -> io::Result<()> {
            self.fail_if(UpdateOperation::SetPermissions)?;
            self.system.set_permissions(file, permissions)
        }

        fn sync_all(&mut self, file: &File) -> io::Result<()> {
            self.fail_if(UpdateOperation::Sync)?;
            self.system.sync_all(file)
        }

        fn persist(&mut self, temporary: tempfile::NamedTempFile, path: &Path) -> Result<()> {
            self.fail_if(UpdateOperation::Persist)?;
            self.system.persist(temporary, path)
        }
    }

    #[test]
    fn update_file_failures_preserve_the_original_document() {
        for operation in [
            UpdateOperation::CreateTemporary,
            UpdateOperation::Write,
            UpdateOperation::Permissions,
            UpdateOperation::SetPermissions,
            UpdateOperation::Sync,
            UpdateOperation::Persist,
        ] {
            let directory = tempfile::tempdir().unwrap();
            let destination = directory.path().join("releases.md");
            fs::write(&destination, "## Unreleased\n").unwrap();
            let mut operations = FailingUpdateIo::new(Some(operation));

            assert!(
                update_file_with(directory.path(), &destination, "## v1\n", &mut operations)
                    .is_err()
            );
            assert_eq!(fs::read_to_string(destination).unwrap(), "## Unreleased\n");
        }
    }

    #[test]
    fn update_file_succeeds_when_all_operations_succeed() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("releases.md");
        fs::write(&destination, "## Unreleased\n").unwrap();
        let mut operations = FailingUpdateIo::new(None);

        update_file_with(directory.path(), &destination, "## v1\n", &mut operations).unwrap();

        assert_eq!(fs::read_to_string(destination).unwrap(), "## v1\n");
    }
}
