use std::{fs, path::Path, sync::Arc};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use comrak::nodes::{Node, NodeCode, NodeValue};

use crate::ConvertOptions;

/// Which optional assets a document needs.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct UsedAssets {
    pub(crate) highlight: bool,
    pub(crate) math:      bool,
}

/// Build the **comrak** options matching the given conversion options.
pub(crate) fn build_comrak_options(options: &ConvertOptions) -> comrak::Options<'static> {
    let mut comrak_options = comrak::Options::default();

    comrak_options.render.r#unsafe = options.allow_unsafe;
    comrak_options.render.hardbreaks = options.hardbreaks;
    comrak_options.render.gfm_quirks = true;
    comrak_options.render.tasklist_classes = true;

    comrak_options.extension.alerts = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.cjk_friendly_emphasis = true;
    comrak_options.extension.description_lists = true;
    comrak_options.extension.footnotes = true;
    comrak_options.extension.front_matter_delimiter = Some(String::from("---"));
    comrak_options.extension.header_id_prefix = Some(String::new());
    comrak_options.extension.multiline_block_quotes = true;
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.subscript = true;
    comrak_options.extension.superscript = true;
    comrak_options.extension.table = true;
    comrak_options.extension.tagfilter = true;
    comrak_options.extension.tasklist = true;

    comrak_options.parse.relaxed_tasklist_matching = true;

    // Without MathJax the math would be shown as raw TeX, so the delimiters are better left alone.
    if options.math {
        comrak_options.extension.math_code = true;
        comrak_options.extension.math_dollars = true;
        comrak_options.extension.math_latex = true;
    }

    if options.embed_images {
        let base_path = options.base_path.unwrap_or_else(|| Path::new("")).to_path_buf();

        comrak_options.extension.image_url_rewriter =
            Some(Arc::new(move |url: &str| match embed_image(base_path.as_path(), url) {
                Some(data_url) => data_url,
                None => url.to_string(),
            }));
    }

    comrak_options
}

/// Find the title of a document, first in its front matter and then in its first level-1 heading.
pub(crate) fn document_title(root: Node) -> Option<String> {
    let mut heading_title = None;

    for node in root.children() {
        match node.data.borrow().value {
            NodeValue::FrontMatter(ref front_matter) => {
                if let Some(title) = front_matter_title(front_matter) {
                    return Some(title);
                }
            },
            NodeValue::Heading(heading) if heading.level == 1 && heading_title.is_none() => {
                let title = node_text(node);

                if !title.is_empty() {
                    heading_title = Some(title);
                }
            },
            _ => (),
        }
    }

    heading_title
}

/// Check which optional assets a document needs.
pub(crate) fn used_assets(root: Node) -> UsedAssets {
    let mut used_assets = UsedAssets::default();

    for node in root.descendants() {
        match node.data.borrow().value {
            NodeValue::CodeBlock(ref code_block) => {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");

                if lang == "math" {
                    used_assets.math = true;
                } else if !lang.is_empty() {
                    // Code blocks without a language are not touched by highlight.js.
                    used_assets.highlight = true;
                }
            },
            NodeValue::Math(..) => used_assets.math = true,
            _ => (),
        }

        if used_assets.highlight && used_assets.math {
            break;
        }
    }

    used_assets
}

/// Read the `title` entry out of a YAML front matter, without pulling in a YAML parser.
fn front_matter_title(front_matter: &str) -> Option<String> {
    for line in front_matter.lines() {
        if let Some(value) = line.trim().strip_prefix("title:") {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|value| value.strip_suffix('\'')))
                .unwrap_or(value);

            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Collect the plain text of a node and of all its descendants.
fn node_text(node: Node) -> String {
    let mut text = String::new();

    for descendant in node.descendants() {
        match descendant.data.borrow().value {
            NodeValue::Text(ref literal) => text.push_str(literal),
            NodeValue::Code(NodeCode {
                ref literal, ..
            }) => text.push_str(literal),
            NodeValue::LineBreak | NodeValue::SoftBreak => text.push(' '),
            _ => (),
        }
    }

    text.trim().to_string()
}

/// Turn a local image into a `data` URL. Remote and absolute URLs are left alone.
fn embed_image(base_path: &Path, url: &str) -> Option<String> {
    if url.is_empty() || url.starts_with('#') || url.starts_with("//") || has_scheme(url) {
        return None;
    }

    let path = base_path.join(percent_decode(url).as_str());
    let mime = image_mime(path.extension()?.to_str()?)?;
    let image = fs::read(path).ok()?;

    Some(format!("data:{mime};base64,{}", BASE64.encode(image)))
}

/// Check whether a URL starts with a scheme. A single letter is not treated as one, so that Windows drive letters still work.
fn has_scheme(url: &str) -> bool {
    match url.find(':') {
        Some(index) => {
            let scheme = &url[..index];

            scheme.len() > 1
                && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        },
        None => false,
    }
}

fn percent_decode(url: &str) -> String {
    let bytes = url.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                match u8::from_str_radix(&url[(index + 1)..(index + 3)], 16) {
                    Ok(byte) => {
                        decoded.push(byte);
                        index += 3;
                    },
                    Err(_) => {
                        decoded.push(b'%');
                        index += 1;
                    },
                }
            },
            byte => {
                decoded.push(byte);
                index += 1;
            },
        }
    }

    String::from_utf8(decoded).unwrap_or_else(|_| url.to_string())
}

fn image_mime(extension: &str) -> Option<&'static str> {
    let mime = match extension.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => return None,
    };

    Some(mime)
}
