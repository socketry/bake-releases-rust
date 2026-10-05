# Bake Releases

Reusable releases.md tasks for Bake, inspired by Samuel Williams's
[Ruby bake-releases](https://github.com/ioquatix/bake-releases) (MIT).

The task functions live under the `releases` module, which becomes the task
namespace. Add this crate as a dependency and reference it from the task binary
so Rust links its registration entries:

```rust,ignore
use bake_releases as _;

bake::Registry::discover()?.run()
```

No per-task registration calls are needed. `#[bake::task]` functions in the
library are collected by `Registry::discover()`.

```sh
cargo bake releases:notes Unreleased
cargo bake releases:update v0.1.0
cargo bake releases:notes v0.1.0 --path releases.md
```

This crate only manages the Markdown release document. Cargo publication and
GitHub release creation are provided by [Bake Cargo](https://github.com/socketry/bake-cargo-rust).

## Document format

Use unindented ATX headings such as `## Unreleased` and `## v0.1.0`. The version
argument matches the complete heading title exactly (including any v prefix).
Optional closing heading markers are supported. Fenced code blocks can contain
example headings. Setext headings, HTML blocks, and headings inside lists or
block quotes are outside this release-document format.

notes returns the body beneath the selected heading until the next heading of
the same or a higher level. Nested sections, whitespace, line endings, and other
Markdown bytes are preserved. Missing and duplicate headings are errors.

update renames exactly one Unreleased heading. It rejects an existing release
heading, missing/duplicate Unreleased sections, and invalid multiline titles.
The rest of the file is preserved. A temporary file in the same directory is
written and synchronized before replacing the original, preserving file permissions.
Existing symlinks are followed. As with other file editors, concurrent external
edits are not merged and replacing a file changes its identity for hard links.

These tasks do not modify Cargo versions, create a new Unreleased section, commit,
tag, or publish releases. The pure `extract_notes` and `update_document` functions
are also available for direct library use.

## Releases

<!-- bake-readme:releases:start -->
See [releases.md](releases.md) for the full release history.

### v0.3.1

- Require Bake 0.18.0 for the shared task registry.

### v0.3.0

- Remove the redundant `bake_releases::releases` module; task adapters are now
  available at the crate root.

### v0.2.4

- Declare compatibility with the Bake 0.x API so task libraries can share one task registry
  when upgrading to crate-derived task namespaces.
<!-- bake-readme:releases:end -->

## See Also

- [bake-releases](https://github.com/socketry/bake-releases-rust) — Reusable releases.md tasks for Bake <!-- bake-readme:package -->

## Contributing

Please open an issue or pull request on
[GitHub](https://github.com/socketry/bake-releases-rust).

### Agent Context

Run `cargo bake agent:context:install` to install shared context and skills.
Read `.agents/context/index.md` to find relevant guides, follow `agents.md` if
present, and apply skills under `.agents/skills/`. See the [Agent Context guide]
for guidance on package context and repository-only instructions.

[Agent Context guide]: https://github.com/socketry/bake-agent-context-rust/blob/main/context/agent-context.md
