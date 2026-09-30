//! Reusable release-document tasks, inspired by Samuel Williams's Ruby
//! `bake-releases`: <https://github.com/ioquatix/bake-releases> (MIT).
//! This implementation preserves Markdown bytes rather than re-rendering them.

mod document;

pub use document::{extract_notes, update_document};

/// Tasks exported by this package register beneath the `releases` namespace.
pub mod releases {
    use bake::{Context, Error, Result};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    use super::{extract_notes, update_document};

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
        let directory = path
            .parent()
            .ok_or_else(|| Error::new("release document has no parent directory"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary.write_all(updated.as_bytes())?;
        temporary
            .as_file()
            .set_permissions(fs::metadata(&path)?.permissions())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&path)
            .map_err(|error| Error::from(error.error))?;
        Ok(())
    }

    /// GitHub release tasks. These require the GitHub CLI to be installed and
    /// authenticated for the repository containing the project root.
    pub mod github {
        use bake::{Context, Error, Result};
        use std::fs;
        use std::io::Write;
        use std::path::PathBuf;
        use std::process::Stdio;

        use super::super::extract_notes;

        /// Create a GitHub Release using notes from the matching releases.md heading.
        /// The remote tag must already exist; use `--draft true` to create a draft.
        #[bake::task]
        pub fn release(
            context: &mut Context,
            tag: String,
            #[bake(default = "releases.md")] path: PathBuf,
            #[bake(default = false)] draft: bool,
        ) -> Result<String> {
            if tag.is_empty() || tag.starts_with('-') || tag.chars().any(char::is_control) {
                return Err(Error::new("tag must be a nonempty GitHub release tag"));
            }

            let path = context.root().join(path);
            let document = fs::read_to_string(&path)
                .map_err(|error| Error::new(format!("{}: {error}", path.display())))?;
            let notes = extract_notes(&document, &tag)?;

            let mut command = context.command("gh");
            command
                .args(["release", "create"])
                .arg(&tag)
                .arg("--title")
                .arg(&tag)
                .args(["--notes-file", "-", "--verify-tag"]);
            if draft {
                command.arg("--draft");
            }

            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            child
                .stdin
                .take()
                .ok_or_else(|| Error::new("failed to open GitHub CLI input"))?
                .write_all(notes.as_bytes())?;
            let output = child.wait_with_output()?;
            if !output.status.success() {
                return Err(Error::new(format!(
                    "GitHub release creation failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )));
            }

            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        }
    }
}
