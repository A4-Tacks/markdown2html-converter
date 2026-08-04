mod cli;

use std::{
    borrow::Cow,
    fs,
    io::{self, Read, Write},
    path::Path,
};

use anyhow::{Context, anyhow};
use cli::*;
use markdown2html_converter::ConvertOptions;

/// The path which stands for the standard input or the standard output.
const STDIO_PATH: &str = "-";

fn main() -> anyhow::Result<()> {
    let args = get_args();

    let from_stdin = args.markdown_path.as_os_str() == STDIO_PATH;

    let file_stem = if from_stdin {
        Cow::from("")
    } else {
        args.markdown_path.file_stem().map(|stem| stem.to_string_lossy()).unwrap_or_default()
    };

    let html_path = match args.output.as_deref() {
        Some(path) if path.as_os_str() == STDIO_PATH => None,
        Some(path) => Some(Cow::from(path)),
        None if from_stdin => None,
        None => {
            let folder_path = args.markdown_path.parent().unwrap();

            Some(Cow::from(folder_path.join(format!("{file_stem}.html"))))
        },
    };

    if let Some(html_path) = html_path.as_deref() {
        match html_path.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() || !args.force {
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
        if args
            .markdown_path
            .metadata()
            .with_context(|| anyhow!("{:?}", args.markdown_path))?
            .is_dir()
        {
            return Err(anyhow!("{:?} is a directory!", args.markdown_path));
        }

        fs::read_to_string(args.markdown_path.as_path())
            .with_context(|| anyhow!("{:?}", args.markdown_path))?
    };

    let css = read_asset(args.css_path.as_deref())?;
    let extra_css = read_asset(args.extra_css_path.as_deref())?;
    let highlight_js = read_asset(args.highlight_js_path.as_deref())?;
    let highlight_css = read_asset(args.highlight_css_path.as_deref())?;
    let mathjax_js = read_asset(args.mathjax_js_path.as_deref())?;

    let options = ConvertOptions {
        title:         args.title.as_deref(),
        default_title: Some(file_stem.as_ref()),
        lang:          args.lang.as_str(),
        theme:         args.theme,
        allow_unsafe:  args.r#unsafe,
        hardbreaks:    !args.no_hardbreaks,
        highlight:     !args.no_highlight,
        math:          !args.no_math,
        cjk_fonts:     !args.no_cjk_fonts,
        minify:        !args.no_minify,
        embed_images:  args.embed_images,
        base_path:     if from_stdin { None } else { args.markdown_path.parent() },
        css:           css.as_deref(),
        extra_css:     extra_css.as_deref(),
        highlight_js:  highlight_js.as_deref(),
        highlight_css: highlight_css.as_deref(),
        mathjax_js:    mathjax_js.as_deref(),
    };

    let html = markdown2html_converter::convert(markdown.as_str(), &options)?;

    match html_path {
        Some(html_path) => {
            fs::write(html_path.as_ref(), html).with_context(|| anyhow!("{html_path:?}"))?
        },
        None => io::stdout().write_all(&html).with_context(|| anyhow!("the standard output"))?,
    }

    Ok(())
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
