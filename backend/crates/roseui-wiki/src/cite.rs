//! Wiki citation helpers.
//!
//! Shared by the REST `/cite` endpoint and the `wiki_cite` MCP tool so the two
//! surfaces produce identical output. Pure functions only — no I/O.

use crate::types::WikiPage;

/// Slugify a Markdown heading into its GitHub-style anchor fragment.
pub fn heading_slug(text: &str) -> String {
    let mut s = String::new();
    let mut prev_dash = false;
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            s.push(c);
            prev_dash = false;
        } else if c.is_whitespace() && !prev_dash {
            s.push('-');
            prev_dash = true;
        }
    }
    s.trim_matches('-').to_string()
}

/// Build a precise citation string for a page.
///
/// * `anchor` — optional heading text or quoted passage. When it matches a
///   Markdown heading, the citation targets `#heading-slug`; otherwise it
///   points at the page root (quoted-text anchoring is intentionally simple:
///   it resolves to the last preceding heading, else root).
/// * `style` — `"markdown"` (default) yields `[title](wiki:<page_id>#<anchor>)`;
///   `"plain"` yields `title (source_ref)` using the page's `source_ref`.
pub fn build_citation(page: &WikiPage, anchor: Option<&str>, style: &str) -> String {
    let anchor_frag = match anchor {
        None => String::new(),
        Some(a) => {
            let trimmed = a.trim();
            if trimmed.is_empty() {
                String::new()
            } else {
                let mut found: Option<String> = None;
                for line in page.content_md.lines() {
                    let line = line.trim_start();
                    if let Some(rest) = line.strip_prefix('#') {
                        let rest = rest.trim_start_matches('#').trim();
                        if rest.eq_ignore_ascii_case(trimmed) {
                            found = Some(heading_slug(rest));
                            break;
                        }
                    }
                }
                match found {
                    Some(slug) => format!("#{slug}"),
                    None => {
                        // Quoted-text anchor: use the last heading before the
                        // matching paragraph, else page root.
                        let heading = page
                            .content_md
                            .lines()
                            .take_while(|l| !l.contains(trimmed))
                            .filter_map(|l| l.trim_start().strip_prefix('#'))
                            .last()
                            .map(|h| heading_slug(h.trim_start_matches('#').trim()))
                            .unwrap_or_default();
                        if heading.is_empty() {
                            String::new()
                        } else {
                            format!("#{heading}")
                        }
                    }
                }
            }
        }
    };

    if style == "plain" {
        if page.source_ref.is_empty() {
            page.title.clone()
        } else {
            format!("{} ({})", page.title, page.source_ref)
        }
    } else {
        format!("[{}](wiki:{}#{})", page.title, page.id, anchor_frag)
    }
}
