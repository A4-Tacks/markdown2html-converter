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

    let used_assets = markdown::used_assets(root, options);
    let has_code = options.highlight && used_assets.highlight;
    let math_mode = options.math.filter(|_| used_assets.math);
    let mermaid_mode = options.mermaid.filter(|_| used_assets.mermaid);

    let mut markdown_html = String::with_capacity(markdown.len() * 3 / 2);

    // Writing into a `String` never fails.
    format_html(root, &comrak_options, &mut markdown_html).unwrap();

    let mut output = Output::new(
        options.minify,
        output_capacity(options, &markdown_html, has_code, math_mode, mermaid_mode),
    );

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

    // The math library has to be defined before the code which uses it runs, so nothing here is deferred or async.
    if let Some(math_mode) = math_mode {
        match math_mode {
            MathMode::MathJaxEmbedded => {
                output.script(MATH_JAX_CONFIG_JS)?;

                match options.mathjax_js {
                    Some(js) => output.script(html_escape::encode_script(js).as_ref())?,
                    None => output.minified_script(MATH_JAX_JS)?,
                }
            },
            MathMode::MathJaxClient => {
                output.script(MATH_JAX_CONFIG_JS)?;
                output.external_script(MATH_JAX_CDN_JS)?;
            },
            MathMode::KatexEmbedded => {
                match options.katex_css {
                    Some(css) => output.style(html_escape::encode_style(css).as_ref())?,
                    None => output.minified_style(KATEX_CSS)?,
                }

                match options.katex_js {
                    Some(js) => output.script(html_escape::encode_script(js).as_ref())?,
                    None => output.minified_script(KATEX_JS)?,
                }
            },
            MathMode::KatexClient => {
                output.stylesheet_link(KATEX_CDN_CSS)?;
                output.external_script(KATEX_CDN_JS)?;
            },
        }
    }

    // Mermaid is loaded here for the same reason as the math library, and it also has to be ready before highlight.js looks at the code blocks.
    if let Some(mermaid_mode) = mermaid_mode {
        match mermaid_mode {
            MermaidMode::Embedded => match options.mermaid_js {
                Some(js) => output.script(html_escape::encode_script(js).as_ref())?,
                None => output.minified_script(MERMAID_JS)?,
            },
            MermaidMode::Client => output.external_script(MERMAID_CDN_JS)?,
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

    // This runs first so that the code blocks it turns into diagrams are gone by the time highlight.js walks the document.
    if mermaid_mode.is_some() {
        output.script(MERMAID_RENDER_JS)?;
    }

    if has_code {
        output.script(HIGHLIGHT_CODE_JS)?;
    }

    if math_mode.is_some_and(MathMode::is_katex) {
        output.script(KATEX_RENDER_JS)?;
    }

    output.digest("</body>")?;
    output.digest("</html>")?;

    Ok(output.into_html())
}

/// Estimate how big the output is going to be. The assets are what makes it big, so only the ones which are going to be written are counted.
fn output_capacity(
    options: &ConvertOptions,
    markdown_html: &str,
    has_code: bool,
    math_mode: Option<MathMode>,
    mermaid_mode: Option<MermaidMode>,
) -> usize {
    let code = if has_code {
        options.highlight_js.map_or(HIGHLIGHT_JS.len(), str::len)
            + options
                .highlight_css
                .map_or(HIGHLIGHT_CSS_LIGHT.len() + HIGHLIGHT_CSS_DARK.len(), str::len)
    } else {
        0
    };

    let math = match math_mode {
        Some(MathMode::MathJaxEmbedded) => options.mathjax_js.map_or(MATH_JAX_JS.len(), str::len),
        Some(MathMode::KatexEmbedded) => {
            options.katex_js.map_or(KATEX_JS.len(), str::len)
                + options.katex_css.map_or(KATEX_CSS.len(), str::len)
        },
        _ => 0,
    };

    let mermaid = match mermaid_mode {
        Some(MermaidMode::Embedded) => options.mermaid_js.map_or(MERMAID_JS.len(), str::len),
        _ => 0,
    };

    let cjk_fonts =
        if options.cjk_fonts { FONT_CJK_CSS.len() + FONT_CJK_MONO_CSS.len() } else { 0 };

    markdown_html.len()
        + PAGE_CSS.len()
        + options.css.map_or(MARKDOWN_CSS.len(), str::len)
        + options.extra_css.map_or(0, str::len)
        + cjk_fonts
        + code
        + math
        + mermaid
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
