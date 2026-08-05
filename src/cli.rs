use std::{path::PathBuf, str::FromStr};

use clap::{CommandFactory, FromArgMatches, Parser, error::ErrorKind};
use concat_with::concat_line;
use markdown2html_converter::{APP_NAME, CARGO_PKG_VERSION, MathMode, Theme};
use terminal_size::terminal_size;

const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

const AFTER_HELP: &str = "Enjoy it! https://magiclen.org";

const APP_ABOUT: &str = concat!(
    "A tool for converting a Markdown file to a single HTML file with built-in CSS and \
     JS.\n\nEXAMPLES:\n",
    concat_line!(prefix "markdown2html-converter ",
        "/path/to/file.md                           # Convert /path/to/file.md to /path/to/file.html, titled \"file\"",
        "/path/to/file.md -o /path/to/output.html   # Convert /path/to/file.md to /path/to/output.html, titled \"output\"",
        "/path/to/file.md -t 'Hello World!'         # Convert /path/to/file.md to /path/to/file.html, titled \"Hello World!\"",
        "/path/to/file.md --theme dark              # Convert /path/to/file.md to /path/to/file.html, always in the dark theme",
        "/path/to/file.md -o - > /path/to/out.html  # Convert /path/to/file.md and write the HTML to the standard output",
        "- -o /path/to/output.html                  # Convert the Markdown from the standard input to /path/to/output.html",
    )
);

#[derive(Debug, Parser)]
#[command(name = APP_NAME)]
#[command(term_width = terminal_size().map(|(width, _)| width.0 as usize).unwrap_or(0))]
#[command(version = CARGO_PKG_VERSION)]
#[command(author = CARGO_PKG_AUTHORS)]
#[command(after_help = AFTER_HELP)]
pub struct CLIArgs {
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your Markdown file, or `-` for the standard input")]
    pub markdown_path: PathBuf,

    #[arg(short, long)]
    #[arg(help = "Specify the title of your HTML file")]
    pub title: Option<String>,

    #[arg(short = 'o', long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your HTML file, or `-` for the standard output")]
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    #[arg(help = "Overwrite the HTML file if it exists")]
    pub force: bool,

    #[arg(short, long, default_value = "en")]
    #[arg(help = "Specify the language of your HTML file")]
    pub lang: String,

    #[arg(long, default_value = "auto", value_parser = parse_theme)]
    #[arg(help = "Specify the color theme of your HTML file [possible values: auto, light, dark]")]
    pub theme: Theme,

    #[arg(long)]
    #[arg(help = "Allow raw HTML and dangerous URLs")]
    pub r#unsafe: bool,

    #[arg(long)]
    #[arg(help = "Embed local images as data URLs")]
    pub embed_images: bool,

    #[arg(long, requires = "embed_images")]
    #[arg(value_hint = clap::ValueHint::DirPath)]
    #[arg(help = "Specify the base directory for relative local images")]
    pub base_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(help = "Do not embed highlight.js")]
    pub no_highlight: bool,

    #[arg(long, value_delimiter = ',', conflicts_with = "no_highlight")]
    #[arg(value_name = "LANGUAGES")]
    #[arg(help = "Specify which code block languages make highlight.js be embedded, separated \
                  by commas, or `any` for all of them [default: the languages of the built-in \
                  highlight.js]")]
    pub highlight_languages: Option<Vec<String>>,

    #[arg(long, default_value_t = MathMode::default(), value_parser = parse_math_mode)]
    #[arg(help = "Specify how the math is rendered [possible values: mathjax-embedded, \
                  mathjax-client, katex-embedded, katex-client]")]
    pub math_mode: MathMode,

    #[arg(long, conflicts_with = "math_mode")]
    #[arg(help = "Do not render math")]
    pub no_math: bool,

    #[arg(long)]
    #[arg(help = "Do not embed CJK fonts")]
    pub no_cjk_fonts: bool,

    #[arg(long)]
    #[arg(help = "Do not render single line breaks as <br>")]
    pub no_hardbreaks: bool,

    #[arg(long)]
    #[arg(help = "Do not minify the output HTML")]
    pub no_minify: bool,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify a CSS file that replaces built-in Markdown styles")]
    pub css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify an extra CSS file to append after all stylesheets")]
    pub extra_css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify a custom highlight.js file. Pass --highlight-languages too when it \
                  supports other languages than the built-in one")]
    pub highlight_js_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify custom CSS for highlight.js code blocks")]
    pub highlight_css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify a custom single-file MathJax bundle")]
    pub mathjax_js_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify a custom KaTeX file")]
    pub katex_js_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify custom CSS for KaTeX")]
    pub katex_css_path: Option<PathBuf>,
}

fn parse_theme(theme: &str) -> Result<Theme, String> {
    Theme::from_str(theme).map_err(|error| error.to_string())
}

fn parse_math_mode(math_mode: &str) -> Result<MathMode, String> {
    MathMode::from_str(math_mode).map_err(|error| error.to_string())
}

pub fn get_args() -> CLIArgs {
    let args = CLIArgs::command();

    let about = format!("{APP_NAME} {CARGO_PKG_VERSION}\n{CARGO_PKG_AUTHORS}\n{APP_ABOUT}");

    let args = args.about(about);

    let matches = args.get_matches();

    let args = match CLIArgs::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(err) => {
            err.exit();
        },
    };

    // clap can require an argument to be present, but not to have a certain value, so the assets which fit only one math mode are checked here.
    if let Some((option, math_mode)) = misplaced_math_asset(&args) {
        CLIArgs::command()
            .error(
                ErrorKind::ArgumentConflict,
                format!("the argument '{option}' can only be used with '--math-mode {math_mode}'"),
            )
            .exit();
    }

    args
}

/// Find an asset option which needs a math mode other than the chosen one.
fn misplaced_math_asset(args: &CLIArgs) -> Option<(&'static str, MathMode)> {
    if args.mathjax_js_path.is_some() && args.math_mode != MathMode::MathJaxEmbedded {
        return Some(("--mathjax-js-path", MathMode::MathJaxEmbedded));
    }

    if args.katex_js_path.is_some() && args.math_mode != MathMode::KatexEmbedded {
        return Some(("--katex-js-path", MathMode::KatexEmbedded));
    }

    if args.katex_css_path.is_some() && args.math_mode != MathMode::KatexEmbedded {
        return Some(("--katex-css-path", MathMode::KatexEmbedded));
    }

    None
}
