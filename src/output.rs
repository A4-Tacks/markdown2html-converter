use html_minifier::{HTMLMinifier, HTMLMinifierError};

/// The output buffer, which either minifies what it is given or keeps it as-is.
pub(crate) enum Output {
    Minified(Box<HTMLMinifier>),
    Plain(Vec<u8>),
}

impl Output {
    #[inline]
    pub(crate) fn new(minify: bool) -> Self {
        if minify { Self::Minified(Box::new(HTMLMinifier::new())) } else { Self::Plain(Vec::new()) }
    }

    #[inline]
    pub(crate) fn digest<S: AsRef<[u8]>>(&mut self, text: S) -> Result<(), HTMLMinifierError> {
        match self {
            Self::Minified(minifier) => minifier.digest(text),
            Self::Plain(buffer) => {
                buffer.extend_from_slice(text.as_ref());

                Ok(())
            },
        }
    }

    /// Append text which is already minified. It saves the minifier from scanning big assets again.
    #[inline]
    pub(crate) fn indigest<S: AsRef<[u8]>>(&mut self, text: S) {
        match self {
            // `indigest` only appends bytes to the output buffer, so there is no safety contract to uphold here.
            Self::Minified(minifier) => unsafe { minifier.indigest(text) },
            Self::Plain(buffer) => buffer.extend_from_slice(text.as_ref()),
        }
    }

    #[inline]
    pub(crate) fn style<S: AsRef<[u8]>>(&mut self, css: S) -> Result<(), HTMLMinifierError> {
        self.digest("<style>")?;
        self.digest(css)?;
        self.digest("</style>")
    }

    #[inline]
    pub(crate) fn script<S: AsRef<[u8]>>(&mut self, js: S) -> Result<(), HTMLMinifierError> {
        self.digest("<script>")?;
        self.digest(js)?;
        self.digest("</script>")
    }

    /// Write a `<script>` element whose content is already minified.
    #[inline]
    pub(crate) fn minified_script<S: AsRef<[u8]>>(
        &mut self,
        js: S,
    ) -> Result<(), HTMLMinifierError> {
        self.digest("<script>")?;
        self.indigest(js);
        self.digest("</script>")
    }

    #[inline]
    pub(crate) fn into_html(self) -> Vec<u8> {
        match self {
            Self::Minified(minifier) => minifier.get_html().to_vec(),
            Self::Plain(buffer) => buffer,
        }
    }
}
