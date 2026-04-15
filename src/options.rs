//! 変換オプション・モード定義
//!
//! [`ConversionMode`] で前処理と変換方針を切り替える。
//! [`ConversionOptions`] はモードとフラグの組み合わせを保持し、
//! `Default` は `balanced` モードを返す。

/// 変換モード。入力 HTML の性質と用途に応じて選択する。
///
/// | モード      | 用途                                  |
/// |-------------|---------------------------------------|
/// | `Balanced`  | 汎用（既定）。読みやすい Markdown      |
/// | `Strict`    | デバッグ・比較。情報保持最大          |
/// | `Minimal`   | LLM 前処理・圧縮。最小限抽出         |
/// | `Semantic`  | SPA / 文書構造重視                   |
/// | `Preserve`  | アーカイブ。元情報最大保持            |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ConversionMode {
    /// 既定。読みやすさと構造保持のバランスをとる。
    #[default]
    Balanced,
    /// 正確性重視。属性削除を最小限にし、デバッグ用途に向く。
    Strict,
    /// 抽出重視。本文と構造の要点のみ抽出する。LLM 前処理に向く。
    Minimal,
    /// 意味重視。アクセシビリティ属性・文書構造を優先する。
    Semantic,
    /// 忠実性重視。変換困難な情報も HTML 断片として保持する。
    Preserve,
}

impl ConversionMode {
    /// モード名を文字列で返す。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Strict => "strict",
            Self::Minimal => "minimal",
            Self::Semantic => "semantic",
            Self::Preserve => "preserve",
        }
    }

    /// 文字列からモードを解析する。大文字小文字を区別しない。
    ///
    /// `std::str::FromStr` を実装しているため、
    /// `"balanced".parse::<ConversionMode>()` でも利用できる。
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

/// 変換オプション。モードとフラグの組み合わせを保持する。
///
/// `Default` は `balanced` モードの推奨設定を返す。
/// 細かいフラグはモードの既定から上書きできる。
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// 変換モード。
    pub mode: ConversionMode,

    // ── 属性保持フラグ ──────────────────────────────────────────────────
    /// `id` 属性を保持するか（アンカーリンク用）。
    pub preserve_ids: bool,
    /// `class` 属性を保持するか。
    pub preserve_classes: bool,
    /// `data-*` カスタム属性を保持するか。
    pub preserve_data_attrs: bool,
    /// `aria-*` アクセシビリティ属性を保持するか。
    pub preserve_aria_attrs: bool,
    /// 未知の属性を保持するか。
    pub preserve_unknown_attrs: bool,

    // ── 前処理フラグ ────────────────────────────────────────────────────
    /// `style`, `class` などの装飾属性を削除するか。
    pub drop_presentation_attrs: bool,
    /// `nav`, `header`, `footer`, `aside` などのシェル要素を除外するか。
    pub drop_interactive_shell: bool,
    /// 意味を持たないラッパー要素をアンラップするか。
    pub unwrap_unknown_wrappers: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self::for_mode(ConversionMode::Balanced)
    }
}

impl ConversionOptions {
    /// 指定したモードの推奨設定でオプションを生成する。
    pub fn for_mode(mode: ConversionMode) -> Self {
        match mode {
            ConversionMode::Balanced => Self {
                mode,
                preserve_ids: true, // アンカー用途のみ
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
                preserve_aria_attrs: true, // 強く保持
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

    /// ビルダー: モードを設定する。
    pub fn mode(mut self, mode: ConversionMode) -> Self {
        self.mode = mode;
        self
    }

    /// ビルダー: `id` 属性の保持を設定する。
    pub fn preserve_ids(mut self, v: bool) -> Self {
        self.preserve_ids = v;
        self
    }

    /// ビルダー: `aria-*` 属性の保持を設定する。
    pub fn preserve_aria_attrs(mut self, v: bool) -> Self {
        self.preserve_aria_attrs = v;
        self
    }

    /// ビルダー: シェル要素（nav/header/footer/aside）の除外を設定する。
    pub fn drop_interactive_shell(mut self, v: bool) -> Self {
        self.drop_interactive_shell = v;
        self
    }
}
