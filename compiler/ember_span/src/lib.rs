//! Source positions: `FileId`, `Span`, and the `SourceMap` that turns a byte
//! offset back into a line and column.
//!
//! Spec: Part XVIII §1 (`ember_span`), `[CMP-2]` — no stage after parsing may
//! report a diagnostic without a source span.
//!
//! Offsets are **byte** offsets into the normalised source text (`[LEX-2]`:
//! CRLF has already become LF and a BOM has already been stripped by the time
//! anything holds a `Span`). Columns are reported in Unicode scalar values,
//! 1-based, because that is what a reader counts when looking at the caret.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

pub mod symbol;
pub use symbol::Symbol;

/// Index of a source file within a [`SourceMap`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileId(pub u32);

impl FileId {
    /// A file id that belongs to no file, for synthesised nodes that must still
    /// carry a span-shaped value. Never resolvable in a `SourceMap`.
    pub const DUMMY: FileId = FileId(u32::MAX);
}

/// A half-open byte range `[start, end)` within one file.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const DUMMY: Span = Span { file: FileId::DUMMY, start: 0, end: 0 };

    pub fn new(file: FileId, start: u32, end: u32) -> Span {
        debug_assert!(start <= end, "span start {start} is past its end {end}");
        Span { file, start, end }
    }

    pub fn is_dummy(self) -> bool {
        self.file == FileId::DUMMY
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// The smallest span covering both. Spans in different files do not merge;
    /// the left operand wins, which keeps a diagnostic pointing somewhere real
    /// rather than at a nonsensical range.
    pub fn to(self, other: Span) -> Span {
        if self.file != other.file {
            return self;
        }
        Span { file: self.file, start: self.start.min(other.start), end: self.end.max(other.end) }
    }

    /// A zero-width span at the start, for "expected something here" carets.
    pub fn shrink_to_start(self) -> Span {
        Span { file: self.file, start: self.start, end: self.start }
    }

    /// A zero-width span at the end, which is where "expected an indented
    /// block" (`[LEX-9]`, `E0004`) wants its caret.
    pub fn shrink_to_end(self) -> Span {
        Span { file: self.file, start: self.end, end: self.end }
    }

    pub fn contains(self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dummy() {
            write!(f, "Span(DUMMY)")
        } else {
            write!(f, "Span({}:{}..{})", self.file.0, self.start, self.end)
        }
    }
}

/// A resolved position: 1-based line and 1-based column in Unicode scalar
/// values.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

/// One source file, with its normalised text and a line index.
pub struct SourceFile {
    pub id: FileId,
    /// Display path, as written on the command line or resolved from an import.
    pub path: PathBuf,
    /// Normalised source: LF endings, no BOM (`[LEX-1]`, `[LEX-2]`).
    pub text: String,
    /// Byte offset of the first character of each line. Always starts with 0.
    line_starts: Vec<u32>,
}

impl SourceFile {
    fn new(id: FileId, path: PathBuf, text: String) -> SourceFile {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        SourceFile { id, path, text, line_starts }
    }

    /// 1-based line number containing `offset`.
    pub fn line_of(&self, offset: u32) -> u32 {
        match self.line_starts.binary_search(&offset) {
            Ok(i) => i as u32 + 1,
            Err(i) => i as u32, // i is the count of line starts <= offset
        }
    }

    /// Byte range of a 1-based line, excluding its terminating newline.
    pub fn line_range(&self, line: u32) -> (u32, u32) {
        let idx = (line as usize).saturating_sub(1);
        let start = self.line_starts.get(idx).copied().unwrap_or(0);
        let end = self
            .line_starts
            .get(idx + 1)
            .map(|&next| next - 1) // drop the '\n'
            .unwrap_or(self.text.len() as u32);
        (start, end.max(start))
    }

    /// The text of a 1-based line, without its newline.
    pub fn line_text(&self, line: u32) -> &str {
        let (start, end) = self.line_range(line);
        &self.text[start as usize..end as usize]
    }

    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    /// Resolve a byte offset to line and column. The column counts Unicode
    /// scalar values, so a caret under an emoji lands under the emoji.
    pub fn line_col(&self, offset: u32) -> LineCol {
        let line = self.line_of(offset).max(1);
        let (line_start, _) = self.line_range(line);
        let clamped = offset.min(self.text.len() as u32) as usize;
        let prefix = &self.text[line_start as usize..clamped];
        LineCol { line, col: prefix.chars().count() as u32 + 1 }
    }
}

/// Every file the compiler has read, keyed by [`FileId`].
#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
    by_path: HashMap<PathBuf, FileId>,
}

impl SourceMap {
    pub fn new() -> SourceMap {
        SourceMap::default()
    }

    /// Add a file whose text is already normalised. Returns the existing id if
    /// the same path was added before, so importing a module twice does not
    /// duplicate its text.
    pub fn add(&mut self, path: impl Into<PathBuf>, text: impl Into<String>) -> FileId {
        let path = path.into();
        if let Some(&existing) = self.by_path.get(&path) {
            return existing;
        }
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, path.clone(), text.into()));
        self.by_path.insert(path, id);
        id
    }

    /// Read a file from disk, normalising line endings and stripping a BOM
    /// (`[LEX-1]`, `[LEX-2]`). Invalid UTF-8 is reported as an error here
    /// rather than reaching the lexer as replacement characters.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<FileId, LoadError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| LoadError::Io(path.to_path_buf(), e))?;
        let text = normalise(&bytes).ok_or_else(|| LoadError::NotUtf8(path.to_path_buf()))?;
        Ok(self.add(path.to_path_buf(), text))
    }

    pub fn file(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }

    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    pub fn files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter()
    }

    /// The source text a span covers.
    pub fn snippet(&self, span: Span) -> &str {
        let file = self.file(span.file);
        &file.text[span.start as usize..span.end as usize]
    }

    pub fn line_col(&self, span: Span) -> LineCol {
        self.file(span.file).line_col(span.start)
    }

    /// `path:line:col`, the prefix of every rendered diagnostic (XIX §6).
    pub fn location(&self, span: Span) -> String {
        if span.is_dummy() {
            return "<unknown>".to_string();
        }
        let file = self.file(span.file);
        let lc = file.line_col(span.start);
        format!("{}:{}:{}", file.path.display(), lc.line, lc.col)
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(PathBuf, std::io::Error),
    /// `[LEX-1]` — reported as `E0001` once it reaches the diagnostic layer.
    NotUtf8(PathBuf),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(p, e) => write!(f, "cannot read {}: {e}", p.display()),
            LoadError::NotUtf8(p) => write!(f, "{} is not valid UTF-8", p.display()),
        }
    }
}

/// Strip a UTF-8 BOM and normalise CRLF and lone CR to LF (`[LEX-2]`).
///
/// Returns `None` if the bytes are not UTF-8, which is `E0001`.
pub fn normalise(bytes: &[u8]) -> Option<String> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).ok()?;
    if !text.contains('\r') {
        return Some(text.to_string());
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            // CRLF and a lone CR both become one LF.
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_with(text: &str) -> (SourceMap, FileId) {
        let mut map = SourceMap::new();
        let id = map.add("test.em", text);
        (map, id)
    }

    #[test]
    fn line_col_is_one_based() {
        let (map, id) = map_with("fn main():\n    println(1)\n");
        let file = map.file(id);
        assert_eq!(file.line_col(0), LineCol { line: 1, col: 1 });
        assert_eq!(file.line_col(11), LineCol { line: 2, col: 1 });
        assert_eq!(file.line_col(15), LineCol { line: 2, col: 5 });
    }

    #[test]
    fn columns_count_scalars_not_bytes() {
        let (map, id) = map_with("# héllo x\n");
        let file = map.file(id);
        // 'x' is byte 9 (é is two bytes) but visually the 9th character.
        assert_eq!(file.text.as_bytes()[9], b'x');
        assert_eq!(file.line_col(9), LineCol { line: 1, col: 9 });
    }

    #[test]
    fn line_text_excludes_the_newline() {
        let (map, id) = map_with("a\nbb\nccc");
        let file = map.file(id);
        assert_eq!(file.line_text(1), "a");
        assert_eq!(file.line_text(2), "bb");
        assert_eq!(file.line_text(3), "ccc");
        assert_eq!(file.line_count(), 3);
    }

    #[test]
    fn offset_at_a_newline_belongs_to_the_line_it_ends() {
        let (map, id) = map_with("ab\ncd\n");
        let file = map.file(id);
        assert_eq!(file.line_of(2), 1); // the '\n' itself
        assert_eq!(file.line_of(3), 2); // first byte of line 2
    }

    #[test]
    fn crlf_and_lone_cr_normalise_to_lf() {
        assert_eq!(normalise(b"a\r\nb\rc\n").unwrap(), "a\nb\nc\n");
    }

    #[test]
    fn bom_is_stripped() {
        assert_eq!(normalise(b"\xEF\xBB\xBFfn").unwrap(), "fn");
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        assert!(normalise(b"\xFF\xFE").is_none());
    }

    #[test]
    fn spans_merge_within_a_file_only() {
        let a = Span::new(FileId(0), 2, 5);
        let b = Span::new(FileId(0), 9, 12);
        assert_eq!(a.to(b), Span::new(FileId(0), 2, 12));
        let other = Span::new(FileId(1), 0, 1);
        assert_eq!(a.to(other), a);
    }

    #[test]
    fn adding_the_same_path_twice_reuses_the_id() {
        let mut map = SourceMap::new();
        let a = map.add("x.em", "one");
        let b = map.add("x.em", "two");
        assert_eq!(a, b);
        assert_eq!(map.file(a).text, "one");
    }
}
