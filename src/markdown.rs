use std::{borrow::Cow, fs, path::Path, sync::Arc};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use comrak::nodes::{Node, NodeCode, NodeValue};
use yaml_rust2::YamlLoader;

use crate::{ConvertOptions, HighlightLanguages, resources::HIGHLIGHT_LANGUAGES};

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

    // Without a renderer the math would be shown as raw TeX, so the delimiters are better left alone.
    if options.math.is_some() {
        comrak_options.extension.math_code = true;
        comrak_options.extension.math_dollars = true;
        comrak_options.extension.math_latex = true;
    }

    if options.embed_images {
        let base_path = options.base_path.unwrap_or(Path::new("")).to_path_buf();

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
                    // A front matter is always the first node, so nothing can override this title any more.
                    heading_title = Some(title);
                    break;
                }
            },
            _ => (),
        }
    }

    heading_title
}

/// Check which optional assets a document needs.
pub(crate) fn used_assets(root: Node, languages: HighlightLanguages) -> UsedAssets {
    let mut used_assets = UsedAssets::default();

    for node in root.descendants() {
        match node.data.borrow().value {
            NodeValue::CodeBlock(ref code_block) => {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");

                if lang == "math" {
                    used_assets.math = true;
                } else if is_highlighted(languages, lang) {
                    // Code blocks without a language, or with one highlight.js cannot handle, are left alone.
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

/// Check whether **highlight.js** is expected to handle a code block language.
fn is_highlighted(languages: HighlightLanguages, lang: &str) -> bool {
    if lang.is_empty() {
        return false;
    }

    match languages {
        HighlightLanguages::BuiltIn => HIGHLIGHT_LANGUAGES
            .lines()
            .filter(|line| !line.starts_with('#'))
            .any(|name| name.eq_ignore_ascii_case(lang)),
        HighlightLanguages::Any => true,
        HighlightLanguages::Only(languages) => {
            languages.iter().any(|name| name.eq_ignore_ascii_case(lang))
        },
    }
}

/// Read the `title` entry out of a YAML front matter.
fn front_matter_title(front_matter: &str) -> Option<String> {
    // The front matter still carries its `---` delimiters, so the title is in the first document.
    let documents = YamlLoader::load_from_str(front_matter).ok()?;
    let title = documents.first()?["title"].as_str()?.trim();

    (!title.is_empty()).then(|| title.to_string())
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

/// Turn a local image into a `data` URL. Only the paths which are relative to the base path are embedded.
fn embed_image(base_path: &Path, url: &str) -> Option<String> {
    if url.is_empty() || url.starts_with(['#', '/', '\\']) || has_scheme(url) {
        return None;
    }

    let (path_url, fragment) =
        url.split_once('#').map_or((url, None), |(path, fragment)| (path, Some(fragment)));
    let path_url = path_url.split_once('?').map_or(path_url, |(path, _)| path);
    let relative_path = percent_decode(path_url);
    let relative_path = Path::new(relative_path.as_ref());

    // A Markdown file should not be able to pull in a file from anywhere on the disk.
    if relative_path.is_absolute() {
        return None;
    }

    let path = base_path.join(relative_path);
    let mime = image_mime(path.extension()?.to_str()?)?;
    let image = fs::read(path).ok()?;

    let mut data_url = String::with_capacity(mime.len() + 13 + image.len().div_ceil(3) * 4);

    data_url.push_str("data:");
    data_url.push_str(mime);
    data_url.push_str(";base64,");

    BASE64.encode_string(image, &mut data_url);

    if let Some(fragment) = fragment {
        data_url.push('#');
        data_url.push_str(fragment);
    }

    Some(data_url)
}

/// Check whether a URL starts with a scheme. A single letter is not treated as one, so that a Windows drive letter is left to the absolute path check.
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

/// Decode the percent-encoded octets of a URL. A URL which has nothing to decode is borrowed as-is.
fn percent_decode(url: &str) -> Cow<'_, str> {
    let bytes = url.as_bytes();

    // `%` is ASCII, so this byte index never falls inside a multi-byte character.
    let Some(start) = url.find('%') else {
        return Cow::Borrowed(url);
    };

    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = start;

    decoded.extend_from_slice(&bytes[..start]);

    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                match hex_octet(bytes[index + 1], bytes[index + 2]) {
                    Some(byte) => {
                        decoded.push(byte);
                        index += 3;
                    },
                    None => {
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

    match String::from_utf8(decoded) {
        Ok(decoded) => Cow::Owned(decoded),
        Err(_) => Cow::Borrowed(url),
    }
}

/// Decode a pair of hexadecimal digits into a byte.
fn hex_octet(high: u8, low: u8) -> Option<u8> {
    Some((hex_digit(high)? << 4) | hex_digit(low)?)
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn image_mime(extension: &str) -> Option<&'static str> {
    const MIMES: [(&str, &str); 9] = [
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        ("svg", "image/svg+xml"),
        ("webp", "image/webp"),
        ("avif", "image/avif"),
        ("bmp", "image/bmp"),
        ("ico", "image/x-icon"),
    ];

    MIMES.into_iter().find(|(name, _)| extension.eq_ignore_ascii_case(name)).map(|(_, mime)| mime)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn a_url_without_percent_encoding_is_borrowed() {
        assert_eq!("images/pic.png", percent_decode("images/pic.png"));
        assert!(matches!(percent_decode("images/pic.png"), Cow::Borrowed(_)));
    }

    #[test]
    fn percent_encoded_octets_are_decoded() {
        assert_eq!("a b.png", percent_decode("a%20b.png"));
        assert_eq!("a/b.png", percent_decode("a%2Fb.png"));
        assert_eq!("圖.png", percent_decode("%E5%9C%96.png"));
    }

    #[test]
    fn a_percent_which_is_not_an_escape_is_kept() {
        // A `%` followed by a non-ASCII character used to be sliced at a byte which is not a character boundary.
        assert_eq!("%圖.png", percent_decode("%圖.png"));
        assert_eq!("%zz.png", percent_decode("%zz.png"));
        // `from_str_radix` used to accept a sign, which made this an escape.
        assert_eq!("%+A.png", percent_decode("%+A.png"));
        assert_eq!("100%", percent_decode("100%"));
        assert_eq!("100%2", percent_decode("100%2"));
    }

    #[test]
    fn only_real_schemes_are_detected() {
        assert!(has_scheme("https://magiclen.org/pic.png"));
        assert!(has_scheme("data:image/png;base64,AAAA"));
        assert!(has_scheme("mailto:len@magiclen.org"));
        assert!(!has_scheme("images/pic.png"));
        assert!(!has_scheme("C:/images/pic.png"));
    }

    #[test]
    fn only_relative_images_are_embedded() {
        let dir = env::temp_dir().join("markdown2html-converter-unit-embed-image");
        let path = dir.join("pic.png");

        fs::create_dir_all(dir.as_path()).unwrap();
        fs::write(path.as_path(), b"an image").unwrap();

        assert_eq!(
            Some(String::from("data:image/png;base64,YW4gaW1hZ2U=")),
            embed_image(dir.as_path(), "pic.png")
        );
        // The file is there, but it may only be reached relatively to the base path.
        assert_eq!(None, embed_image(dir.as_path(), path.to_str().unwrap()));
        assert_eq!(None, embed_image(dir.as_path(), "https://magiclen.org/pic.png"));
        assert_eq!(None, embed_image(dir.as_path(), "//magiclen.org/pic.png"));
        assert_eq!(None, embed_image(dir.as_path(), "#anchor"));
    }
}
