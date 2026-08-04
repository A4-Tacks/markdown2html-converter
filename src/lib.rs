/*!
# Markdown to HTML Converter

Markdown to HTML Converter is a free tool for converting a Markdown file to a single HTML file with built-in CSS and JS.
*/

mod markdown;
mod options;
mod output;
mod resources;

use std::borrow::Cow;

use comrak::{Arena, format_html, parse_document};
pub use html_minifier::HTMLMinifierError as ConvertError;
pub use options::*;

use crate::{output::Output, resources::*};

/// The name of this application.
pub const APP_NAME: &str = "Markdown to HTML Converter";

/// The version of this application.
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

const FALLBACK_TITLE: &str = "Untitled";

/// Convert a Markdown text to a single HTML file with built-in CSS and JS.
///
/// ```rust
/// use markdown2html_converter::ConvertOptions;
///
/// let html = markdown2html_converter::convert(
///     "# Hello world!",
///     &ConvertOptions::default(),
/// )
/// .unwrap();
///
/// assert!(
///     String::from_utf8(html)
///         .unwrap()
///         .contains("<title>Hello world!</title>")
/// );
/// ```
pub fn convert(markdown: &str, options: &ConvertOptions) -> Result<Vec<u8>, ConvertError> {
    let comrak_options = markdown::build_comrak_options(options);

    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &comrak_options);

    let title = match options.title.map(str::trim).filter(|title| !title.is_empty()) {
        Some(title) => Cow::from(title),
        None => match markdown::document_title(root) {
            Some(title) => Cow::from(title),
            None => Cow::from(
                options
                    .default_title
                    .map(str::trim)
                    .filter(|title| !title.is_empty())
                    .unwrap_or(FALLBACK_TITLE),
            ),
        },
    };

    let used_assets = markdown::used_assets(root);
    let has_code = options.highlight && used_assets.highlight;
    let has_math = options.math && used_assets.math;

    let mut markdown_html = String::with_capacity(markdown.len() * 3 / 2);

    // Writing into a `String` never fails.
    format_html(root, &comrak_options, &mut markdown_html).unwrap();

    let mut output = Output::new(options.minify);

    output.digest("<!DOCTYPE html>")?;
    output.digest(html_element(options))?;

    output.digest("<head>")?;
    output.digest("<meta charset=UTF-8>")?;
    output.digest(
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, \
         shrink-to-fit=no\">",
    )?;
    output.digest(format!(
        "<meta name=\"generator\" content=\"{APP_NAME} {CARGO_PKG_VERSION} by magiclen.org\">"
    ))?;
    output.digest("<title>")?;
    output.digest(html_escape::encode_text(title.as_ref()).as_ref())?;
    output.digest("</title>")?;

    output.style(PAGE_CSS)?;

    match options.theme {
        Theme::Auto => (),
        Theme::Light => output.style(PAGE_CSS_LIGHT)?,
        Theme::Dark => output.style(PAGE_CSS_DARK)?,
    }

    match options.css {
        Some(css) => output.style(html_escape::encode_style(css).as_ref())?,
        None => output.style(match options.theme {
            Theme::Auto => MARKDOWN_CSS,
            Theme::Light => MARKDOWN_CSS_LIGHT,
            Theme::Dark => MARKDOWN_CSS_DARK,
        })?,
    }

    if options.cjk_fonts {
        output.style(FONT_CJK_CSS)?;
        output.style(FONT_CJK_MONO_CSS)?;
    }

    if has_code {
        match options.highlight_css {
            Some(css) => output.style(html_escape::encode_style(css).as_ref())?,
            None => match options.theme {
                Theme::Auto => output.minified_styles([
                    "@media (prefers-color-scheme:light){",
                    HIGHLIGHT_CSS_LIGHT,
                    "}@media (prefers-color-scheme:dark){",
                    HIGHLIGHT_CSS_DARK,
                    "}",
                ])?,
                Theme::Light => output.minified_style(HIGHLIGHT_CSS_LIGHT)?,
                Theme::Dark => output.minified_style(HIGHLIGHT_CSS_DARK)?,
            },
        }

        match options.highlight_js {
            Some(js) => output.script(html_escape::encode_script(js).as_ref())?,
            None => output.minified_script(HIGHLIGHT_JS)?,
        }
    }

    if has_math {
        output.script(MATH_JAX_CONFIG_JS)?;

        match options.mathjax_js {
            Some(js) => output.script(html_escape::encode_script(js).as_ref())?,
            None => output.minified_script(MATH_JAX_JS)?,
        }
    }

    if let Some(css) = options.extra_css {
        output.style(html_escape::encode_style(css).as_ref())?;
    }

    output.digest("</head>")?;

    output.digest("<body>")?;
    output.digest("<article class=\"markdown-body\">")?;
    output.digest(&markdown_html)?;
    output.digest("</article>")?;

    if has_code {
        output.script(HIGHLIGHT_CODE_JS)?;
    }

    output.digest("</body>")?;
    output.digest("</html>")?;

    Ok(output.into_html())
}

fn html_element(options: &ConvertOptions) -> String {
    let mut html_element = String::from("<html");

    if !options.lang.is_empty() {
        html_element.push_str(" lang=\"");
        html_element.push_str(html_escape::encode_double_quoted_attribute(options.lang).as_ref());
        html_element.push('"');
    }

    if let Some(theme) = options.theme.attribute_value() {
        html_element.push_str(" data-theme=\"");
        html_element.push_str(theme);
        html_element.push('"');
    }

    html_element.push('>');

    html_element
}
