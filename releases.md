# Releases

## Unreleased

- Remove the redundant `bake_releases::releases` module; task adapters are now
  available at the crate root.

## v0.2.4

- Declare compatibility with the Bake 0.x API so task libraries can share one task registry
  when upgrading to crate-derived task namespaces.

## v0.2.3

- Extract and update release sections using parsed Markdown headings.

## v0.2.2

- Create or update GitHub Releases after successful crates.io publication.
- Resolve the local task crate during version updates.

## v0.2.1

- Switch the runtime dependency from `socketry-bake` to `bake` 0.17.0.

- Add Bake Agent Context tasks to the project's development executable.
- Link the shared Rust context guidance from the README.

## v0.2.0

- Keep this crate focused on release-document extraction and editing; GitHub Release creation now lives in `bake-cargo`.

## v0.1.0

- Move the reusable releases.md task library into its own repository.
- Add an explicit task for creating GitHub Releases from release notes.
