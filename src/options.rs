//! Conversion options and mode definitions
//!
//! [`ConversionMode`] selects the preprocessing and conversion policy.
//! [`ConversionOptions`] holds a mode together with its flags, and
//! `Default` returns the `balanced` mode.

/// Conversion mode. Choose one to suit the input HTML and what it is for.
///
/// | Mode        | Intended for                                  |
/// |-------------|-----------------------------------------------|
/// | `Balanced`  | General purpose (default); readable Markdown   |
/// | `Strict`    | Debugging and comparison; maximum retention    |
/// | `Minimal`   | LLM preprocessing and compaction; bare extract |
/// | `Semantic`  | SPAs; document structure first                 |
/// | `Preserve`  | Archiving; retains as much as possible         |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ConversionMode {
    /// Default. Balances readability against structural fidelity.
    #[default]
    Balanced,
    /// Accuracy first. Removes as few attributes as possible; suits debugging.
    Strict,
    /// Extraction first. Keeps only the body text and the essential structure;
    /// suits LLM preprocessing.
    Minimal,
    /// Meaning first. Favours accessibility attributes and document structure.
    Semantic,
    /// Fidelity first. Keeps even hard-to-convert content, as HTML fragments.
    Preserve,
}

impl ConversionMode {
    /// Returns the mode's name as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Strict => "strict",
            Self::Minimal => "minimal",
            Self::Semantic => "semantic",
            Self::Preserve => "preserve",
        }
    }

    /// Parses a mode from a string, case-insensitively.
    ///
    /// `std::str::FromStr` is implemented as well, so
    /// `"balanced".parse::<ConversionMode>()` works too.
    pub fn parse_mode(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl std::str::FromStr for ConversionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "balanced" => Ok(Self::Balanced),
            "strict" => Ok(Self::Strict),
            "minimal" => Ok(Self::Minimal),
            "semantic" => Ok(Self::Semantic),
            "preserve" => Ok(Self::Preserve),
            other => Err(format!("unknown conversion mode: {other}")),
        }
    }
}

impl std::fmt::Display for ConversionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Conversion options: a mode together with its flags.
///
/// `Default` returns the recommended settings for the `balanced` mode.
/// Individual flags can be overridden from the mode's defaults.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// The conversion mode.
    pub mode: ConversionMode,

    // ── Attribute-retention flags ──────────────────────────────────────────
    /// Whether to keep `id` attributes. When enabled, an
    /// `<a id="...">...</a>` anchor is emitted for an element carrying a
    /// non-empty `id`, as the element's leading content: after a heading
    /// marker, a list marker or a quote prefix. Where the anchor goes for links
    /// and code blocks is described on the
    /// [options page](https://nabbisen.github.io/mdka-rs/api/options.html#preserve_ids).
    pub preserve_ids: bool,
    /// Inert. Markdown has no syntax for class attributes, so this has no effect.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    pub preserve_classes: bool,
    /// Inert. Markdown has no syntax for data-* attributes, so this has no effect.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    pub preserve_data_attrs: bool,
    /// Inert. Markdown has no syntax for aria-* attributes, so this has no effect.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    pub preserve_aria_attrs: bool,
    /// Inert. Markdown has no syntax for unknown attributes, so this has no effect.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    pub preserve_unknown_attrs: bool,

    // ── Preprocessing flags ────────────────────────────────────────────────
    /// Inert. No code path emits presentational attributes at all, so this has
    /// no effect.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    pub drop_presentation_attrs: bool,
    /// Whether to drop shell elements such as `nav`, `header`, `footer` and
    /// `aside`.
    pub drop_interactive_shell: bool,
    /// No effect today. Unwrapping a wrapper element (`<div>`, `<section>`,
    /// `<article>`, `<main>`) removes the tag but keeps the paragraph break
    /// it stood for, so the tag's removal alone leaves nothing left for this
    /// option to change. Not deprecated: unlike the fields above, this one is
    /// inert only because `<div>` has no Markdown form today — a future mode
    /// that preserves raw HTML wrappers would make it observable again. See
    /// the [options page](https://nabbisen.github.io/mdka-rs/api/options.html#unwrap_unknown_wrappers).
    pub unwrap_unknown_wrappers: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self::for_mode(ConversionMode::Balanced)
    }
}

impl ConversionOptions {
    /// Builds the options with the recommended settings for the given mode.
    #[allow(deprecated)]
    pub fn for_mode(mode: ConversionMode) -> Self {
        match mode {
            ConversionMode::Balanced => Self {
                mode,
                preserve_ids: true, // anchors only
                preserve_classes: false,
                preserve_data_attrs: false,
                preserve_aria_attrs: true,
                preserve_unknown_attrs: false,
                drop_presentation_attrs: true,
                drop_interactive_shell: false,
                unwrap_unknown_wrappers: false,
            },
            ConversionMode::Strict => Self {
                mode,
                preserve_ids: true,
                preserve_classes: true,
                preserve_data_attrs: true,
                preserve_aria_attrs: true,
                preserve_unknown_attrs: true,
                drop_presentation_attrs: false,
                drop_interactive_shell: false,
                unwrap_unknown_wrappers: false,
            },
            ConversionMode::Minimal => Self {
                mode,
                preserve_ids: false,
                preserve_classes: false,
                preserve_data_attrs: false,
                preserve_aria_attrs: false,
                preserve_unknown_attrs: false,
                drop_presentation_attrs: true,
                drop_interactive_shell: true,
                unwrap_unknown_wrappers: true,
            },
            ConversionMode::Semantic => Self {
                mode,
                preserve_ids: true,
                preserve_classes: false,
                preserve_data_attrs: false,
                preserve_aria_attrs: true, // retained strongly
                preserve_unknown_attrs: false,
                drop_presentation_attrs: true,
                drop_interactive_shell: false,
                unwrap_unknown_wrappers: true,
            },
            ConversionMode::Preserve => Self {
                mode,
                preserve_ids: true,
                preserve_classes: true,
                preserve_data_attrs: true,
                preserve_aria_attrs: true,
                preserve_unknown_attrs: true,
                drop_presentation_attrs: false,
                drop_interactive_shell: false,
                unwrap_unknown_wrappers: false,
            },
        }
    }

    /// Builder: sets the mode.
    pub fn mode(mut self, mode: ConversionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder: sets whether `id` attributes are kept.
    pub fn preserve_ids(mut self, v: bool) -> Self {
        self.preserve_ids = v;
        self
    }

    /// Builder: sets whether `aria-*` attributes are kept.
    #[deprecated(
        since = "2.2.0",
        note = "no effect: Markdown has no attribute syntax. See https://nabbisen.github.io/mdka-rs/api/options.html"
    )]
    #[allow(deprecated)]
    pub fn preserve_aria_attrs(mut self, v: bool) -> Self {
        self.preserve_aria_attrs = v;
        self
    }

    /// Builder: sets whether shell elements (nav/header/footer/aside) are dropped.
    pub fn drop_interactive_shell(mut self, v: bool) -> Self {
        self.drop_interactive_shell = v;
        self
    }
}
