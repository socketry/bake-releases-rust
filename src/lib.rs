// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! Reusable release-document tasks, inspired by Samuel Williams's Ruby
//! `bake-releases`: <https://github.com/ioquatix/bake-releases> (MIT).
//! This implementation preserves Markdown bytes rather than re-rendering them.
mod document;

pub use document::{extract_notes, update_document};

use bake::{Context, Error, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn parent_directory(path: &Path) -> Result<&Path> {
    path.parent()
        .ok_or_else(|| Error::new("release document has no parent directory"))
}

fn persist_file(temporary: tempfile::NamedTempFile, path: &Path) -> Result<()> {
    temporary
        .persist(path)
        .map(|_| ())
        .map_err(|error| Error::from(error.error))
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
    let directory = parent_directory(&path)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(updated.as_bytes())?;
    temporary
        .as_file()
        .set_permissions(fs::metadata(&path)?.permissions())?;
    temporary.as_file().sync_all()?;
    persist_file(temporary, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_directory_requires_a_path_parent() {
        assert!(parent_directory(Path::new("/")).is_err());
    }

    #[test]
    fn persist_file_reports_a_missing_destination_directory() {
        let directory = tempfile::tempdir().unwrap();
        let temporary = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
        let destination = directory.path().join("missing/releases.md");

        assert!(persist_file(temporary, &destination).is_err());
    }
}
