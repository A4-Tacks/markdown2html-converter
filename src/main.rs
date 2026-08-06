mod cli;

use std::{
    borrow::Cow,
    fs,
    io::{self, Read, Write},
    path::Path,
};

use anyhow::{Context, anyhow};
use cli::*;
use markdown2html_converter::{ConvertOptions, HighlightLanguages};
use same_file::is_same_file;

/// The path which stands for the standard input or the standard output.
const STDIO_PATH: &str = "-";

fn main() -> anyhow::Result<()> {
    let args = get_args();

    let from_stdin = args.markdown_path.as_os_str() == STDIO_PATH;

    if !from_stdin
        && args
            .markdown_path
            .metadata()
            .with_context(|| anyhow!("{:?}", args.markdown_path))?
            .is_dir()
    {
        return Err(anyhow!("{:?} is a directory!", args.markdown_path));
    }

    // A base path which cannot be resolved would otherwise leave every image unembedded without a word.
    if let Some(base_path) = args.base_path.as_deref()
        && !base_path.metadata().with_context(|| anyhow!("{base_path:?}"))?.is_dir()
    {
        return Err(anyhow!("{base_path:?} is not a directory!"));
    }

    let input_file_stem = if from_stdin {
        Cow::from("")
    } else {
        args.markdown_path.file_stem().map(|stem| stem.to_string_lossy()).unwrap_or_default()
    };

    let default_title = match args.output.as_deref() {
        Some(path) if path.as_os_str() != STDIO_PATH => {
            path.file_stem().map(|stem| stem.to_string_lossy()).unwrap_or_default()
        },
        _ => input_file_stem.clone(),
    };

    let html_path = match args.output.as_deref() {
        Some(path) if path.as_os_str() == STDIO_PATH => None,
        Some(path) => Some(Cow::from(path)),
        None if from_stdin => None,
        None => {
            let folder_path = args.markdown_path.parent().unwrap_or(Path::new(""));

            Some(Cow::from(folder_path.join(format!("{input_file_stem}.html"))))
        },
    };

    if let Some(html_path) = html_path.as_deref() {
        if !from_stdin && paths_resolve_to_same_file(args.markdown_path.as_path(), html_path)? {
            return Err(anyhow!("the input and output paths refer to the same file"));
        }

        match html_path.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    return Err(anyhow!("{html_path:?} is a directory!"));
                }

                if !args.force {
                    return Err(anyhow!("{html_path:?} exists!"));
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error).with_context(|| anyhow!("{html_path:?}")),
        }
    }

    let markdown = if from_stdin {
        let mut markdown = String::new();

        io::stdin().read_to_string(&mut markdown).with_context(|| anyhow!("the standard input"))?;

        markdown
    } else {
        fs::read_to_string(args.markdown_path.as_path())
            .with_context(|| anyhow!("{:?}", args.markdown_path))?
    };

    let css = read_asset(args.css_path.as_deref())?;
    let extra_css = read_asset(args.extra_css_path.as_deref())?;
    let highlight_js = read_asset(args.highlight_js_path.as_deref())?;
    let highlight_css = read_asset(args.highlight_css_path.as_deref())?;
    let mathjax_js = read_asset(args.mathjax_js_path.as_deref())?;
    let katex_js = read_asset(args.katex_js_path.as_deref())?;
    let katex_css = read_asset(args.katex_css_path.as_deref())?;
    let mermaid_js = read_asset(args.mermaid_js_path.as_deref())?;

    let highlight_languages: Option<Vec<&str>> = args
        .highlight_languages
        .as_deref()
        .map(|languages| languages.iter().map(String::as_str).collect());

    let options = ConvertOptions {
        title:               args.title.as_deref(),
        default_title:       Some(default_title.as_ref()),
        lang:                args.lang.as_str(),
        theme:               args.theme,
        allow_unsafe:        args.r#unsafe,
        hardbreaks:          !args.no_hardbreaks,
        highlight:           !args.no_highlight,
        highlight_languages: match highlight_languages.as_deref() {
            // `any` is a reserved name which no highlight.js language uses.
            Some(languages) if languages.iter().any(|l| l.eq_ignore_ascii_case("any")) => {
                HighlightLanguages::Any
            },
            Some(languages) => HighlightLanguages::Only(languages),
            None => HighlightLanguages::BuiltIn,
        },
        math:                if args.no_math { None } else { Some(args.math_mode) },
        mermaid:             if args.no_mermaid { None } else { Some(args.mermaid_mode) },
        cjk_fonts:           !args.no_cjk_fonts,
        minify:              !args.no_minify,
        embed_images:        args.embed_images,
        base_path:           args.base_path.as_deref().or(if from_stdin {
            None
        } else {
            args.markdown_path.parent()
        }),
        css:                 css.as_deref(),
        extra_css:           extra_css.as_deref(),
        highlight_js:        highlight_js.as_deref(),
        highlight_css:       highlight_css.as_deref(),
        mathjax_js:          mathjax_js.as_deref(),
        katex_js:            katex_js.as_deref(),
        katex_css:           katex_css.as_deref(),
        mermaid_js:          mermaid_js.as_deref(),
    };

    let html = markdown2html_converter::convert(markdown.as_str(), &options)?;

    match html_path {
        Some(html_path) => {
            let mut open_options = fs::OpenOptions::new();

            open_options.write(true);

            if args.force {
                open_options.create(true).truncate(true);
            } else {
                open_options.create_new(true);
            }

            let mut html_file =
                open_options.open(html_path.as_ref()).with_context(|| anyhow!("{html_path:?}"))?;

            html_file.write_all(&html).with_context(|| anyhow!("{html_path:?}"))?;
        },
        None => io::stdout().write_all(&html).with_context(|| anyhow!("the standard output"))?,
    }

    Ok(())
}

fn paths_resolve_to_same_file(first: &Path, second: &Path) -> io::Result<bool> {
    match is_same_file(first, second) {
        Ok(same_file) => Ok(same_file),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn read_asset(path: Option<&Path>) -> anyhow::Result<Option<String>> {
    match path {
        Some(path) => {
            let asset = fs::read_to_string(path).with_context(|| anyhow!("{path:?}"))?;

            Ok(Some(asset))
        },
        None => Ok(None),
    }
}
