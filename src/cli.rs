use std::{path::PathBuf, str::FromStr};

use clap::{CommandFactory, FromArgMatches, Parser};
use concat_with::concat_line;
use markdown2html_converter::{APP_NAME, CARGO_PKG_VERSION, Theme};
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
    #[arg(help = "Force to output if the HTML file exists")]
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
    #[arg(help = "Embed local images as `data` URLs")]
    pub embed_images: bool,

    #[arg(long)]
    #[arg(help = "Not allow to use highlight.js")]
    pub no_highlight: bool,

    #[arg(long)]
    #[arg(help = "Not allow to use MathJax")]
    pub no_math: bool,

    #[arg(long)]
    #[arg(help = "Not allow to use CJK fonts")]
    pub no_cjk_fonts: bool,

    #[arg(long)]
    #[arg(help = "Not treat a single line break as a line break")]
    pub no_hardbreaks: bool,

    #[arg(long)]
    #[arg(help = "Not minify the output HTML")]
    pub no_minify: bool,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your custom CSS file")]
    pub css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of an extra CSS file to append")]
    pub extra_css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your custom highlight.js file")]
    pub highlight_js_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your custom CSS file for highlight.js code blocks")]
    pub highlight_css_path: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Specify the path of your custom single MathJax file")]
    pub mathjax_js_path: Option<PathBuf>,
}

fn parse_theme(theme: &str) -> Result<Theme, String> {
    Theme::from_str(theme).map_err(|error| error.to_string())
}

pub fn get_args() -> CLIArgs {
    let args = CLIArgs::command();

    let about = format!("{APP_NAME} {CARGO_PKG_VERSION}\n{CARGO_PKG_AUTHORS}\n{APP_ABOUT}");

    let args = args.about(about);

    let matches = args.get_matches();

    match CLIArgs::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(err) => {
            err.exit();
        },
    }
}
