use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    str::FromStr,
};

/// The color theme of the output HTML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Follow the `prefers-color-scheme` media feature.
    #[default]
    Auto,
    /// Always use the light theme.
    Light,
    /// Always use the dark theme.
    Dark,
}

impl Theme {
    /// The value of the `data-theme` attribute of the `<html>` element, if the theme is fixed.
    #[inline]
    pub const fn attribute_value(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::Light => Some("light"),
            Self::Dark => Some("dark"),
        }
    }
}

impl Display for Theme {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Light => "light",
            Self::Dark => "dark",
        })
    }
}

impl FromStr for Theme {
    type Err = UnknownTheme;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(UnknownTheme),
        }
    }
}

/// The error of parsing a [`Theme`] from a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownTheme;

impl Display for UnknownTheme {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("the theme should be one of `auto`, `light` and `dark`")
    }
}

impl std::error::Error for UnknownTheme {}

/// Options for [`convert`](crate::convert).
#[derive(Debug, Clone)]
pub struct ConvertOptions<'a> {
    /// The title of the HTML file. When it is `None`, the title is looked up in the front matter and then in the first level-1 heading.
    pub title:         Option<&'a str>,
    /// The title to use when no title can be found in the Markdown file.
    pub default_title: Option<&'a str>,
    /// The `lang` attribute of the `<html>` element. An empty string omits the attribute.
    pub lang:          &'a str,
    /// The color theme.
    pub theme:         Theme,
    /// Allow raw HTML and dangerous URLs.
    pub allow_unsafe:  bool,
    /// Treat a single line break as a `<br>`.
    pub hardbreaks:    bool,
    /// Embed **highlight.js** when the Markdown file has code blocks with a language.
    pub highlight:     bool,
    /// Embed **MathJax** when the Markdown file has math.
    pub math:          bool,
    /// Embed the CJK fonts.
    pub cjk_fonts:     bool,
    /// Minify the output HTML.
    pub minify:        bool,
    /// Embed local images as `data` URLs.
    pub embed_images:  bool,
    /// The directory that relative paths in the Markdown file are resolved against.
    pub base_path:     Option<&'a Path>,
    /// A stylesheet which replaces the built-in one.
    pub css:           Option<&'a str>,
    /// A stylesheet which is appended after all the other stylesheets.
    pub extra_css:     Option<&'a str>,
    /// A script which replaces the built-in **highlight.js**.
    pub highlight_js:  Option<&'a str>,
    /// A stylesheet which replaces the built-in **highlight.js** theme.
    pub highlight_css: Option<&'a str>,
    /// A script which replaces the built-in **MathJax**.
    pub mathjax_js:    Option<&'a str>,
}

impl Default for ConvertOptions<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            title:         None,
            default_title: None,
            lang:          "en",
            theme:         Theme::default(),
            allow_unsafe:  false,
            hardbreaks:    true,
            highlight:     true,
            math:          true,
            cjk_fonts:     true,
            minify:        true,
            embed_images:  false,
            base_path:     None,
            css:           None,
            extra_css:     None,
            highlight_js:  None,
            highlight_css: None,
            mathjax_js:    None,
        }
    }
}
