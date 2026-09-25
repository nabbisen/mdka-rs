//! Conversion options and mode definitions
//!
//! [`ConversionMode`] selects the preprocessing and conversion policy.
//! [`ConversionOptions`] holds a mode together with its flags, and
//! `Default` returns the `balanced` mode.

/// Conversion mode. There are two, and they make two behaviours.
///
/// | Mode       | Converts                                                      |
/// |------------|---------------------------------------------------------------|
/// | `Balanced` | General purpose (default); readable Markdown                  |
/// | `Minimal`  | LLM preprocessing and compaction: body text and structure only |
///
/// The three former aliases of `Balanced` — `Strict`, `Semantic` and `Preserve` —
/// were removed in 3.0. They produced identical output and could not differ, so
/// nothing that used one converts differently; the name only has to change. See
/// the [Conversion Modes](https://nabbisen.github.io/mdka-rs/api/modes.html) page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ConversionMode {
    /// Default. Balances readability against structural fidelity.
    #[default]
    Balanced,
    /// Extraction first. Keeps only the body text and the essential structure;
    /// suits LLM preprocessing.
    Minimal,
}

impl ConversionMode {
    /// Returns the mode's name as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Minimal => "minimal",
        }
    }

    /// Parses a mode from a string, case-insensitively.
    ///
    /// `std::str::FromStr` is implemented as well, so
    /// `"balanced".parse::<ConversionMode>()` works too -- and **that is the one
    /// to use**: it reports *why* a name was rejected, including that a mode
    /// removed in 3.0 (`"strict"`, `"semantic"`, `"preserve"`) was removed rather
    /// than never existed. This function discards that message and answers `None`
    /// for both.
    ///
    /// Deprecated since 3.0.0. It was never warned about before, so it is not
    /// removed in this release.
    #[deprecated(
        since = "3.0.0",
        note = "use `str::parse`, which reports why a name was rejected; `parse_mode` discards that message"
    )]
    pub fn parse_mode(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

/// Mode names that existed until 2.9.0 and were removed in 3.0. All three were
/// aliases of `Balanced`.
///
/// `FromStr` is the one place the CLI, the Node.js binding and `parse_mode` all
/// route a mode string through, so this table gives every one of them the right
/// message — including a Rust caller who reads a mode from a config file, whom
/// no compiler warning could ever reach. A removed name is **obsolete**, not
/// **wrong**, and the message has to say so.
const REMOVED_MODES: [&str; 3] = ["strict", "semantic", "preserve"];

impl std::str::FromStr for ConversionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = s.to_ascii_lowercase();
        match name.as_str() {
            "balanced" => Ok(Self::Balanced),
            "minimal" => Ok(Self::Minimal),
            removed if REMOVED_MODES.contains(&removed) => Err(format!(
                "conversion mode '{removed}' was removed in 3.0; it was an alias of 'balanced'. \
                 Use 'balanced'."
            )),
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

    /// Whether to keep `id` attributes. When enabled, an
    /// `<a id="...">...</a>` anchor is emitted for an element carrying a
    /// non-empty `id`, as the element's leading content: after a heading
    /// marker, a list marker or a quote prefix. Where the anchor goes for links
    /// and code blocks is described on the
    /// [options page](https://nabbisen.github.io/mdka-rs/api/options.html#preserve_ids).
    pub preserve_ids: bool,
    /// Whether to drop shell elements such as `nav`, `header`, `footer` and
    /// `aside`.
    pub drop_interactive_shell: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self::for_mode(ConversionMode::Balanced)
    }
}

impl ConversionOptions {
    /// Builds the options with the recommended settings for the given mode.
    pub fn for_mode(mode: ConversionMode) -> Self {
        match mode {
            ConversionMode::Balanced => Self {
                mode,
                preserve_ids: true, // anchors only
                drop_interactive_shell: false,
            },
            ConversionMode::Minimal => Self {
                mode,
                preserve_ids: false,
                drop_interactive_shell: true,
            },
        }
    }

    /// Whether a wrapper element (`<div>`, `<section>`, `<article>`, `<main>`)
    /// has its tag removed, keeping its children and the paragraph break it stood
    /// for.
    ///
    /// Not an option. It could never change the output -- Markdown has no
    /// wrapper element to show the difference -- and the field that exposed it,
    /// `unwrap_unknown_wrappers`, was removed in 3.0. It stays a property of the
    /// mode, exactly as it was: `Minimal` unwrapped and `Balanced` rendered, so
    /// each mode converts precisely as it did in 2.9.0 without anyone having to
    /// prove that the two paths agree on every possible input.
    pub(crate) fn unwraps_wrappers(&self) -> bool {
        self.mode == ConversionMode::Minimal
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

    /// Builder: sets whether shell elements (nav/header/footer/aside) are dropped.
    pub fn drop_interactive_shell(mut self, v: bool) -> Self {
        self.drop_interactive_shell = v;
        self
    }
}
