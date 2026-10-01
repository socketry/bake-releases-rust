// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

use bake::{Error, Result};
use socketry_markdown::{ParseOptions, mdast::Node, to_mdast};
use std::ops::Range;

struct Heading {
    level: u8,
    title: String,
    title_range: Option<Range<usize>>,
    start: usize,
    body_start: usize,
}

fn after_heading_line(document: &str, offset: usize) -> usize {
    let Some(rest) = document.get(offset..) else {
        return offset;
    };

    if rest.starts_with("\r\n") {
        offset + 2
    } else if rest.starts_with('\r') || rest.starts_with('\n') {
        offset + 1
    } else {
        offset
    }
}

fn plain_title_range(document: &str, node: &Node) -> Option<Range<usize>> {
    let Node::Heading(heading) = node else {
        return None;
    };
    let [Node::Text(text)] = heading.children.as_slice() else {
        return None;
    };
    let position = text.position.as_ref()?;
    let title = chomp_line_ending(&text.value);
    let mut end = position.end.offset;
    if document.get(..end)?.ends_with("\r\n") {
        end -= 2;
    } else if document.get(..end)?.ends_with('\r') || document.get(..end)?.ends_with('\n') {
        end -= 1;
    }
    let range = position.start.offset..end;
    (document.get(range.clone())? == title).then_some(range)
}

fn chomp_line_ending(text: &str) -> &str {
    text.strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .or_else(|| text.strip_suffix('\r'))
        .unwrap_or(text)
}

// Release documents use unindented ATX headings. Restricting section boundaries
// to document children excludes headings inside lists and block quotes.
fn headings(document: &str) -> Result<Vec<Heading>> {
    let root = to_mdast(document, &ParseOptions::default())
        .map_err(|error| Error::new(format!("could not parse release Markdown: {error}")))?;
    let children = root
        .children()
        .ok_or_else(|| Error::new("parsed release Markdown has no document children"))?;
    let mut headings = Vec::new();

    for node in children {
        let Node::Heading(heading) = node else {
            continue;
        };
        let Some(position) = heading.position.as_ref() else {
            continue;
        };
        let start = position.start.offset;

        // Replacements operate on single-line ATX release headings. The parser
        // has already distinguished headings from fences and other Markdown.
        if position.start.column != 1
            || !document
                .get(start..)
                .is_some_and(|rest| rest.starts_with('#'))
        {
            continue;
        }

        headings.push(Heading {
            level: heading.depth,
            title: chomp_line_ending(&node.text_content()).to_owned(),
            title_range: plain_title_range(document, node),
            start,
            body_start: after_heading_line(document, position.end.offset),
        });
    }

    Ok(headings)
}

fn unique_heading<'headings>(
    headings: &'headings [Heading],
    title: &str,
) -> Result<&'headings Heading> {
    let mut matching = headings.iter().filter(|heading| heading.title == title);
    let heading = matching
        .next()
        .ok_or_else(|| Error::new(format!("release heading {title:?} not found")))?;
    if matching.next().is_some() {
        return Err(Error::new(format!(
            "release heading {title:?} is ambiguous"
        )));
    }
    Ok(heading)
}

/// Extract the body under an exact ATX heading, retaining nested sections and
/// original Markdown bytes. Missing or duplicate headings are errors.
///
/// Headings must be unindented, e.g. `## v0.1.0`. Setext headings, HTML blocks,
/// and headings inside block quotes or lists are outside this document format.
pub fn extract_notes<'document>(document: &'document str, version: &str) -> Result<&'document str> {
    let headings = headings(document)?;
    let heading = unique_heading(&headings, version)?;
    let end = headings
        .iter()
        .find(|candidate| candidate.start > heading.start && candidate.level <= heading.level)
        .map_or(document.len(), |candidate| candidate.start);
    Ok(&document[heading.body_start..end])
}

/// Replace exactly one `Unreleased` heading without reformatting the document.
/// Existing version headings and multiline version strings are rejected.
pub fn update_document(document: &str, version: &str) -> Result<String> {
    if version.is_empty()
        || version.trim() != version
        || version.chars().any(char::is_control)
        || version.contains('#')
        || version == "Unreleased"
    {
        return Err(Error::new(
            "version must be a nonempty single-line heading other than Unreleased, without # characters",
        ));
    }
    let headings = headings(document)?;
    if headings.iter().any(|heading| heading.title == version) {
        return Err(Error::new(format!(
            "release heading {version:?} already exists"
        )));
    }
    let heading = unique_heading(&headings, "Unreleased")?;
    let title_range = heading
        .title_range
        .as_ref()
        .ok_or_else(|| Error::new("Unreleased heading must have a plain text title"))?;
    let mut output = document.to_owned();
    output.replace_range(title_range.clone(), version);
    Ok(output)
}
