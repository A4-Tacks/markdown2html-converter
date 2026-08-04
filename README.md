Markdown to HTML Converter
====================

[![CI](https://github.com/magiclen/markdown2html-converter/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/markdown2html-converter/actions/workflows/ci.yml)

Markdown to HTML Converter is a free tool for converting a Markdown file to a single HTML file with built-in CSS and JS.

## Help

```
EXAMPLES:
markdown2html-converter /path/to/file.md                           # Convert /path/to/file.md to /path/to/file.html, titled "file"
markdown2html-converter /path/to/file.md -o /path/to/output.html   # Convert /path/to/file.md to /path/to/output.html, titled "output"
markdown2html-converter /path/to/file.md -t 'Hello World!'         # Convert /path/to/file.md to /path/to/file.html, titled "Hello World!"
markdown2html-converter /path/to/file.md --theme dark              # Convert /path/to/file.md to /path/to/file.html, always in the dark theme
markdown2html-converter /path/to/file.md -o - > /path/to/out.html  # Convert /path/to/file.md and write the HTML to the standard output
markdown2html-converter - -o /path/to/output.html                  # Convert the Markdown from the standard input to /path/to/output.html

Usage: markdown2html-converter [OPTIONS] <MARKDOWN_PATH>

Arguments:
  <MARKDOWN_PATH>  Specify the path of your Markdown file, or `-` for the standard input

Options:
  -t, --title <TITLE>                            Specify the title of your HTML file
  -o, --output <OUTPUT>                          Specify the path of your HTML file, or `-` for the standard output
  -f, --force                                    Overwrite the HTML file if it exists
  -l, --lang <LANG>                              Specify the language of your HTML file [default: en]
      --theme <THEME>                            Specify the color theme of your HTML file [possible values: auto, light, dark] [default: auto]
      --unsafe                                   Allow raw HTML and dangerous URLs
      --embed-images                             Embed local images as data URLs
      --base-path <BASE_PATH>                    Specify the base directory for relative local images
      --no-highlight                             Do not embed highlight.js
      --no-math                                  Do not embed MathJax
      --no-cjk-fonts                             Do not embed CJK fonts
      --no-hardbreaks                            Do not render single line breaks as <br>
      --no-minify                                Do not minify the output HTML
      --css-path <CSS_PATH>                      Specify a CSS file that replaces built-in Markdown styles
      --extra-css-path <EXTRA_CSS_PATH>          Specify an extra CSS file to append after all stylesheets
      --highlight-js-path <HIGHLIGHT_JS_PATH>    Specify a custom highlight.js file
      --highlight-css-path <HIGHLIGHT_CSS_PATH>  Specify custom CSS for highlight.js code blocks
      --mathjax-js-path <MATHJAX_JS_PATH>        Specify a custom single-file MathJax bundle
  -h, --help                                     Print help
  -V, --version                                  Print version
```

## Title

The title of the output HTML file is looked up in this order.

1. The `-t` (`--title`) option.
2. The `title` entry of the YAML front matter of the Markdown file.
3. The first level-1 heading of the Markdown file.
4. The file name of the Markdown file.

When an output file is explicitly specified with `-o` (`--output`), its file name is used as the fallback title instead.

## Custom Assets

`--css-path` replaces the built-in Markdown stylesheet. `--extra-css-path` is appended after every generated stylesheet, so it can override any built-in or custom styles. `--highlight-js-path`, `--highlight-css-path`, and `--mathjax-js-path` replace their respective built-in assets.

Custom CSS and JavaScript are embedded in the generated HTML. Use only trusted files.

`--base-path` controls how relative local image paths are resolved by `--embed-images`. It defaults to the Markdown file's directory and is required to choose another directory when reading Markdown from the standard input.

## Dependency

Markdown is converted to HTML by the [comrak](https://crates.io/crates/comrak) crate, with the GFM extensions (tables, task lists, footnotes, autolinks, strikethrough, [alerts](https://github.com/orgs/community/discussions/16925)) enabled. The default stylesheet (the CSS file) is from [sindresorhus/github-markdown-css](https://github.com/sindresorhus/github-markdown-css).

If ` ``` ` is used with a language in the input Markdown file, the [highlight.js](https://highlightjs.org/) will be automatically embedded in the output HTML file. The preset supported languages are listed below.

* Apache
* Bash
* C
* C#
* C++
* CSS
* Diff
* Dockerfile
* Go
* GraphQL
* HTML, XML
* JSON
* Java
* JavaScript
* Kotlin
* Less
* Lua
* Makefile
* Markdown
* Nginx
* Objective-C
* PHP
* PHP Template
* Perl
* Plain Text
* Python
* Python REPL
* R
* Ruby
* Rust
* SCSS
* SQL
* Shell
* Swift
* TOML, INI
* TypeScript
* Visual Basic .NET
* WebAssembly
* YAML

If math is used in the input Markdown file, the [MathJax](https://www.mathjax.org/) will be automatically embedded in the output HTML file. The supported syntaxes are listed below.

| Syntax | Result |
| --- | --- |
| `$E = mc^2$` | inline math |
| `$$x = y$$` | display math |
| `\(a + b\)` | inline math |
| `\[a + b\]` | display math |
| ` ```math ` block | display math |

The default **MathJax** is the [tex-mml-svg](https://docs.mathjax.org/en/latest/web/components/combined.html#tex-mml-svg) configuration file. It draws math with inline SVG paths, so the output HTML file needs no web font and works offline.

## Themes

By default, the output HTML file follows the `prefers-color-scheme` media feature. Use `--theme light` or `--theme dark` to always use one of them.

## Offline Usage

Everything but the CJK fonts is embedded in the output HTML file. Add `--no-cjk-fonts` to make the output HTML file completely self-contained, and `--embed-images` to inline the local images it refers to.

## Library

This crate can also be used as a library.

```rust
use markdown2html_converter::ConvertOptions;

let html = markdown2html_converter::convert("# Hello world!", &ConvertOptions::default()).unwrap();
```

## A Markdown Example

[The Markdown File](https://github.com/magiclen/markdown2html-converter/blob/master/example.md)

[The HTML File](https://jsfiddle.net/magiclen/jgs324w0/latest)

## License

[MIT](LICENSE)
