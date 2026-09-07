//! Tokens, keywords and punctuation (Part II).

use std::fmt;

use ember_span::{Span, Symbol};

/// One token. `[II.7]` — `INDENT`/`DEDENT`/`NEWLINE` are real tokens and doc
/// comments are tokens; ordinary comments are not.
#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn is(&self, kind: &TokenKind) -> bool {
        &self.kind == kind
    }

    pub fn keyword(&self) -> Option<Kw> {
        match self.kind {
            TokenKind::Keyword(k) => Some(k),
            _ => None,
        }
    }

    pub fn ident(&self) -> Option<Symbol> {
        match self.kind {
            TokenKind::Ident(s) | TokenKind::RawIdent(s) => Some(s),
            _ => None,
        }
    }

    pub fn punct(&self) -> Option<Punct> {
        match self.kind {
            TokenKind::Punct(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    /// End of a logical line at bracket depth zero (`[LEX-5]`).
    Newline,
    Indent,
    Dedent,
    Eof,

    Ident(Symbol),
    /// `r#match` — a keyword used as a name (`[LEX-14]`).
    RawIdent(Symbol),
    Keyword(Kw),
    /// Lexed as a keyword so that using one is `E0005` rather than a confusing
    /// parse error.
    Reserved(Reserved),

    Lit(Lit),
    Punct(Punct),

    /// `## text` — attaches to the next declaration.
    DocComment(String),
    /// `#! name "value"` on the first line of a file only.
    Directive { name: Symbol, value: String },

    /// An unrecognised character. The lexer never fails fatally (`[II.7]`).
    Error,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Newline => f.write_str("end of line"),
            TokenKind::Indent => f.write_str("an indented block"),
            TokenKind::Dedent => f.write_str("the end of a block"),
            TokenKind::Eof => f.write_str("end of file"),
            TokenKind::Ident(s) => write!(f, "`{s}`"),
            TokenKind::RawIdent(s) => write!(f, "`r#{s}`"),
            TokenKind::Keyword(k) => write!(f, "`{}`", k.as_str()),
            TokenKind::Reserved(r) => write!(f, "`{}`", r.as_str()),
            TokenKind::Lit(_) => f.write_str("a literal"),
            TokenKind::Punct(p) => write!(f, "`{}`", p.as_str()),
            TokenKind::DocComment(_) => f.write_str("a doc comment"),
            TokenKind::Directive { .. } => f.write_str("a directive"),
            TokenKind::Error => f.write_str("an unrecognised character"),
        }
    }
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
pub enum Lit {
    /// `[LEX-16]` — an integer with no suffix is "untyped" and takes its type
    /// from context. The value is kept at full width; range checking against
    /// the chosen type happens in `ember_typeck` (`E2010`).
    Int { value: u128, suffix: Option<IntSuffix> },
    /// `[LEX-17]` — an unsuffixed float is "untyped float"; absent context it
    /// becomes `f32`. Kept as `f64` so no precision is lost before the type is
    /// known.
    Float { value: f64, suffix: Option<FloatSuffix> },
    Char(char),
    /// Type `str` with static region (`[LEX-20]`).
    Str(String),
    /// `b"…"` — `Span[u8]`, static region.
    Bytes(Vec<u8>),
    /// `c"…"` — NUL-terminated, static region. The trailing NUL is added at
    /// codegen, not stored here.
    CStr(Vec<u8>),
    /// `f"…{e:spec}…"` — the parser sub-parses each expression span.
    FStr(Vec<FStrPart>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum FStrPart {
    /// Literal text, with escapes and `{{`/`}}` already resolved.
    Text(String),
    /// An interpolated expression. `span` covers the expression source inside
    /// the braces, so diagnostics from sub-parsing point at the real file.
    Expr { span: Span, format_spec: Option<String> },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IntSuffix {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    Isize,
    Usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FloatSuffix {
    F16,
    F32,
    F64,
}

impl IntSuffix {
    pub fn from_str(text: &str) -> Option<IntSuffix> {
        Some(match text {
            "i8" => IntSuffix::I8,
            "i16" => IntSuffix::I16,
            "i32" => IntSuffix::I32,
            "i64" => IntSuffix::I64,
            "i128" => IntSuffix::I128,
            "u8" => IntSuffix::U8,
            "u16" => IntSuffix::U16,
            "u32" => IntSuffix::U32,
            "u64" => IntSuffix::U64,
            "u128" => IntSuffix::U128,
            "isize" => IntSuffix::Isize,
            "usize" => IntSuffix::Usize,
            _ => return None,
        })
    }
}

impl FloatSuffix {
    pub fn from_str(text: &str) -> Option<FloatSuffix> {
        Some(match text {
            "f16" => FloatSuffix::F16,
            "f32" => FloatSuffix::F32,
            "f64" => FloatSuffix::F64,
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

macro_rules! keywords {
    ($enum_name:ident, $( $variant:ident => $text:literal ),* $(,)?) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub enum $enum_name { $($variant),* }

        impl $enum_name {
            pub fn as_str(self) -> &'static str {
                match self { $( $enum_name::$variant => $text ),* }
            }

            pub fn from_str(text: &str) -> Option<$enum_name> {
                Some(match text { $( $text => $enum_name::$variant, )* _ => return None })
            }

            pub const ALL: &'static [$enum_name] = &[ $($enum_name::$variant),* ];
        }
    };
}

// Reserved keywords, v1 (Part II §4).
keywords! { Kw,
    And => "and", As => "as", Break => "break", Class => "class",
    Comptime => "comptime", Const => "const", Continue => "continue",
    Defer => "defer", Dyn => "dyn", Elif => "elif", Else => "else",
    Enum => "enum", Extend => "extend", Extern => "extern", False => "false",
    Fn => "fn", For => "for", From => "from", If => "if",
    Implements => "implements", Import => "import", In => "in",
    Interface => "interface", Is => "is", Match => "match", Mut => "mut",
    Not => "not", Open => "open", Or => "or", Override => "override",
    Owned => "owned", Pass => "pass", Pub => "pub", Ref => "ref",
    Return => "return", SelfValue => "self", SelfType => "Self",
    Static => "static", Struct => "struct", Super => "super", True => "true",
    Unsafe => "unsafe", Virtual => "virtual", Void => "void", Where => "where",
    While => "while", With => "with",
}

// Reserved for future use: lexed as keywords, `E0005` if used (Part II §4).
keywords! { Reserved,
    Actor => "actor", Async => "async", Await => "await", Macro => "macro",
    Yield => "yield", Move => "move", Trait => "trait", Type => "type",
    Union => "union", Loop => "loop", Unless => "unless",
}

/// `[LEX-15]` — keywords only in the grammatical positions named in Part III,
/// ordinary identifiers everywhere else. The lexer emits these as `Ident`; the
/// parser recognises them by text where the grammar allows.
// `let` is added to the list: `[CLS-9]` gives `let name: T` a meaning as an
// immutable field, but Part II §4 lists it neither as a keyword nor as
// contextual. See docs/spec-errata.md ERR-004.
pub const CONTEXTUAL_KEYWORDS: &[&str] =
    &["abstract", "final", "lazy", "test", "bench", "let"];

// ---------------------------------------------------------------------------
// Punctuation
// ---------------------------------------------------------------------------

macro_rules! puncts {
    ($( $variant:ident => $text:literal ),* $(,)?) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Punct { $($variant),* }

        impl Punct {
            pub fn as_str(self) -> &'static str {
                match self { $( Punct::$variant => $text ),* }
            }
            /// Every spelling, longest first, so that a linear scan performs
            /// maximal munch (`[LEX-21]`).
            pub fn by_length_desc() -> Vec<(&'static str, Punct)> {
                let mut all = vec![ $( ($text, Punct::$variant) ),* ];
                all.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
                all
            }
        }
    };
}

puncts! {
    // three characters
    StarStarEq => "**=", ShlEq => "<<=", ShrEq => ">>=", DotDotEq => "..=",
    // two characters
    StarStar => "**", EqEq => "==", NotEq => "!=", LtEq => "<=", GtEq => ">=",
    Shl => "<<", Shr => ">>", PlusEq => "+=", MinusEq => "-=", StarEq => "*=",
    SlashEq => "/=", PercentEq => "%=", AmpEq => "&=", PipeEq => "|=",
    CaretEq => "^=", DotDot => "..", Arrow => "->", FatArrow => "=>",
    QuestionDot => "?.", ColonColon => "::",
    // one character
    Plus => "+", Minus => "-", Star => "*", Slash => "/", Percent => "%",
    // `;` is absent from Part II §6's operator table but required by
    // Part III's `simple_stmt`, `array_type` and `array_lit`.
    // See docs/spec-errata.md ERR-003.
    Semi => ";",
    Amp => "&", Pipe => "|", Caret => "^", Tilde => "~", Lt => "<", Gt => ">",
    Eq => "=", Dot => ".", Comma => ",", Colon => ":", At => "@",
    Question => "?", Bang => "!", Hash => "#",
    LParen => "(", RParen => ")", LBracket => "[", RBracket => "]",
    LBrace => "{", RBrace => "}",
}

impl Punct {
    pub fn is_open_bracket(self) -> bool {
        matches!(self, Punct::LParen | Punct::LBracket | Punct::LBrace)
    }

    pub fn is_close_bracket(self) -> bool {
        matches!(self, Punct::RParen | Punct::RBracket | Punct::RBrace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_v1_keyword_list_is_the_one_in_the_spec() {
        // Part II §4 lists seven keywords on each of six rows and five on the
        // seventh. A count that drifts means the table and this enum disagree.
        assert_eq!(Kw::ALL.len(), 47);
        assert_eq!(Kw::from_str("owned"), Some(Kw::Owned));
        assert_eq!(Kw::from_str("Self"), Some(Kw::SelfType));
        assert_eq!(Kw::from_str("self"), Some(Kw::SelfValue));
        assert_eq!(Kw::from_str("abstract"), None, "abstract is contextual [LEX-15]");
    }

    #[test]
    fn reserved_words_are_separate_from_live_keywords() {
        assert_eq!(Reserved::from_str("async"), Some(Reserved::Async));
        assert_eq!(Kw::from_str("async"), None);
        assert_eq!(Reserved::from_str("type"), Some(Reserved::Type));
    }

    #[test]
    fn punctuation_is_ordered_longest_first_for_maximal_munch() {
        let all = Punct::by_length_desc();
        let lengths: Vec<usize> = all.iter().map(|(t, _)| t.len()).collect();
        assert!(lengths.windows(2).all(|w| w[0] >= w[1]));
        assert_eq!(all[0].0.len(), 3);
    }
}
