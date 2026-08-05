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
      --highlight-languages <LANGUAGES>          Specify which code block languages make highlight.js be embedded, separated by commas, or `any` for all of them [default: the languages of the built-in highlight.js]
      --math-mode <MATH_MODE>                    Specify how the math is rendered [possible values: mathjax-embedded, mathjax-client, katex-embedded, katex-client] [default: katex-client]
      --no-math                                  Do not render math
      --no-cjk-fonts                             Do not embed CJK fonts
      --no-hardbreaks                            Do not render single line breaks as <br>
      --no-minify                                Do not minify the output HTML
      --css-path <CSS_PATH>                      Specify a CSS file that replaces built-in Markdown styles
      --extra-css-path <EXTRA_CSS_PATH>          Specify an extra CSS file to append after all stylesheets
      --highlight-js-path <HIGHLIGHT_JS_PATH>    Specify a custom highlight.js file. Pass --highlight-languages too when it supports other languages than the built-in one
      --highlight-css-path <HIGHLIGHT_CSS_PATH>  Specify custom CSS for highlight.js code blocks
      --mathjax-js-path <MATHJAX_JS_PATH>        Specify a custom single-file MathJax bundle
      --katex-js-path <KATEX_JS_PATH>            Specify a custom KaTeX file
      --katex-css-path <KATEX_CSS_PATH>          Specify custom CSS for KaTeX
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

`--css-path` replaces the built-in Markdown stylesheet. `--extra-css-path` is appended after every generated stylesheet, so it can override any built-in or custom styles. `--highlight-js-path`, `--highlight-css-path`, `--mathjax-js-path`, `--katex-js-path`, and `--katex-css-path` replace their respective built-in assets. Each of the math assets can only be used with the mode it belongs to, so `--mathjax-js-path` needs `--math-mode mathjax-embedded`, and the two KaTeX ones need `--math-mode katex-embedded`.

`--highlight-js-path` does not change which languages make **highlight.js** be embedded, which is still the language list of the built-in build. When a custom build supports other languages, pass `--highlight-languages` as well, either with the languages you use or with `any`.

Custom CSS and JavaScript are embedded in the generated HTML. Use only trusted files.

`--base-path` controls how relative local image paths are resolved by `--embed-images`. It defaults to the Markdown file's directory and is required to choose another directory when reading Markdown from the standard input. It can only be used together with `--embed-images`.

Only the image paths which are relative to the base path are embedded. Remote URLs, `data` URLs, and absolute paths such as `/pictures/pic.png` or `C:\pictures\pic.png` are left untouched.

## Dependency

Markdown is converted to HTML by the [comrak](https://crates.io/crates/comrak) crate, with the GFM extensions (tables, task lists, footnotes, autolinks, strikethrough, [alerts](https://github.com/orgs/community/discussions/16925)) enabled. The default stylesheet (the CSS file) is from [sindresorhus/github-markdown-css](https://github.com/sindresorhus/github-markdown-css).

If ` ``` ` is used with one of the languages below in the input Markdown file, the [highlight.js](https://highlightjs.org/) will be automatically embedded in the output HTML file. Any other language, such as ` ```mermaid `, leaves it out, because the built-in **highlight.js** could not do anything with it anyway. Use `--highlight-languages` to change that list. Aliases such as `js` and `c++` work too, and the matching ignores the case.

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

If math is used in the input Markdown file, a math renderer is added to the output HTML file. The supported syntaxes are listed below.

| Syntax | Result |
| --- | --- |
| `$E = mc^2$` | inline math |
| `$$x = y$$` | display math |
| `` $`a + b`$ `` | inline math |
| `\(a + b\)` | inline math |
| `\[a + b\]` | display math |
| ` ```math ` block | display math |

## Math Modes

`--math-mode` picks the renderer and how it travels with the output HTML file. The sizes below are what each mode adds to a file which contains math.

| Mode | Added | Works offline | Notes |
| --- | --- | --- | --- |
| `katex-client` (default) | ~0 | no | Loads [KaTeX](https://katex.org/) from a CDN |
| `katex-embedded` | ~630 KB | yes | Embeds **KaTeX**, its stylesheet, and its fonts |
| `mathjax-client` | ~0 | no | Loads [MathJax](https://www.mathjax.org/) from a CDN |
| `mathjax-embedded` | ~1.85 MB | yes | Embeds the whole **MathJax** bundle |

The built-in **MathJax** is the [tex-mml-svg](https://docs.mathjax.org/en/latest/web/components/combined.html#tex-mml-svg) configuration file, which draws math with inline SVG paths and therefore needs no web font. **KaTeX** needs its own fonts, so `katex-embedded` carries them inside its stylesheet as `data` URLs.

Use `--no-math` to leave the math alone. The delimiters are then not treated as math at all, so a `$` in the text stays a `$`.

## Themes

By default, the output HTML file follows the `prefers-color-scheme` media feature. Use `--theme light` or `--theme dark` to always use one of them.

## Offline Usage

The default math mode loads **KaTeX** from a CDN, so a file which contains math needs the network. Pick an embedded math mode to avoid that.

```bash
markdown2html-converter /path/to/file.md --math-mode katex-embedded --no-cjk-fonts --embed-images
```

`--math-mode katex-embedded` (or `mathjax-embedded`) keeps the math renderer inside the file, `--no-cjk-fonts` drops the only other CDN reference, and `--embed-images` inlines the local images it refers to. The result is a completely self-contained HTML file.

## Library

This crate can also be used as a library.

```rust
use markdown2html_converter::ConvertOptions;

let html = markdown2html_converter::convert("# Hello world!", &ConvertOptions::default()).unwrap();
```

## Updating the Built-in Assets

Two files under `resources` are generated rather than taken from upstream as-is, so they need a step of their own.

`resources/katex.min.css` is the stylesheet of **KaTeX** with its woff2 fonts inlined as `data` URLs and the woff and ttf fallbacks dropped. To move to another **KaTeX** version, copy the `katex.min.js` of its `dist` directory over `resources/katex.min.js`, then build the stylesheet out of the same `dist` directory.

```bash
node tools/build-katex-css.js /path/to/katex/dist > resources/katex.min.css
```

`resources/highlight-languages.txt` lists the languages, aliases included, which `resources/highlight.min.js` registers. Regenerate it whenever the **highlight.js** build changes, and update the language list in this README to match.

```bash
node tools/list-highlight-languages.js resources/highlight.min.js
```

The CDN URLs of the client math modes are pinned in `src/resources.rs` and have to be moved together with the files above, so that every mode renders the same math.

## A Markdown Example

[The Markdown File](https://github.com/magiclen/markdown2html-converter/blob/master/example.md)

[The HTML File](https://jsfiddle.net/magiclen/jgs324w0/latest)

## License

[MIT](LICENSE)
