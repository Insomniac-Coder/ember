## `std.string` — Part XV's text module. `String` and `str` are compiler-known
## until the library can declare them; this module holds what can already be
## written.

## `[TXT-10]` (ODR-029) — why `s.parse[T]()` failed: the text was empty, held
## something that is not part of a `T` literal, or named a value outside `T`.
## A unit-only enum, so it is `Copy`, `Eq` and `Debug`.
pub enum ParseError:
    Empty
    Invalid
    Overflow

## `[TXT-2]` — invalid UTF-8 when viewing foreign or raw bytes as text.
pub enum Utf8Error:
    Invalid

## `[TXT-10]` — the iterators a `str` gives, each holding a view of the text,
## so every item it gives borrows the text too. `s.chars()` gives the
## characters, `s.char_indices()` each with its byte offset, `s.bytes()` the
## bytes, `s.lines()` the lines without their `\n` or `\r\n` (a last line
## without one is a line; a newline at the end starts none), `s.split(sep)`
## the parts between each `sep` (Python's `split(sep)`: an empty part between
## two in a row; an empty `sep` panics, as Python raises), and
## `s.split_whitespace()` the runs of non-white-space characters (Python's
## `split()` with no argument).
extend str:
    pub fn chars(self) -> Chars:
        return Chars(text = self, at = 0)

    pub fn char_indices(self) -> CharIndices:
        return CharIndices(text = self, at = 0)

    pub fn bytes(self) -> Bytes:
        return Bytes(text = self, at = 0)

    pub fn lines(self) -> Lines:
        return Lines(rest = self)

    @borrows(self, sep)
    pub fn split(self, sep: str) -> Split:
        if sep.len() == 0:
            panic("split: the separator is empty")
        return Split(rest = self, sep = sep, done = false)

    pub fn split_whitespace(self) -> SplitWhitespace:
        return SplitWhitespace(rest = self)

## Python's white space for `str.split()`: the characters `str.isspace()`
## accepts. `@inline`: `split_whitespace` asks it of every character, and
## MSVC kept it a call (1.5x the C loop); the spaces past ASCII are a
## function of their own.
@inline
fn is_split_space(c: char) -> bool:
    code = c as u32
    if code <= 0x20:
        # 0x09 to 0x0D, 0x1C to 0x1F and 0x20: bits of one word.
        return (0x1F0003E00 >> (code as int)) & 1 == 1
    if code < 0x85:
        return false
    return is_wide_split_space(code)

fn is_wide_split_space(code: u32) -> bool:
    return (code == 0x85 or code == 0xA0 or code == 0x1680 or (code >= 0x2000 and code <= 0x200A)
        or code == 0x2028 or code == 0x2029 or code == 0x202F or code == 0x205F or code == 0x3000)

@derive(Copy)
pub struct Chars:
    text: str
    at: int

extend Chars implements Iterator:
    type Item = char

    ## `next_char` decodes the character at `at` and moves `at` past it;
    ## `at` stays on a boundary.
    fn next(mut self) -> Option[char]:
        if self.at >= self.text.len():
            return None
        return Some(self.text.next_char(self.at))

@derive(Copy)
pub struct CharIndices:
    text: str
    at: int

extend CharIndices implements Iterator:
    type Item = (int, char)

    fn next(mut self) -> Option[(int, char)]:
        if self.at >= self.text.len():
            return None
        here = self.at
        return Some((here, self.text.next_char(self.at)))

@derive(Copy)
pub struct Bytes:
    text: str
    at: int

extend Bytes implements Iterator:
    type Item = u8

    fn next(mut self) -> Option[u8]:
        if self.at >= self.text.len():
            return None
        b = self.text.as_bytes()[self.at]
        self.at += 1
        return Some(b)

@derive(Copy)
pub struct Lines:
    rest: str

extend Lines implements Iterator:
    type Item = str

    ## The text is copied out of the field first, so a line borrows the
    ## text, not this iterator.
    fn next(mut self) -> Option[str]:
        rest = self.rest
        if rest.len() == 0:
            return None
        match rest.find("\n"):
            Some(at):
                line = rest[..at]
                self.rest = rest[at + 1..]
                if line.ends_with("\r"):
                    return Some(line[..line.len() - 1])
                return Some(line)
            None:
                self.rest = rest[rest.len()..]
                return Some(rest)

@derive(Copy)
pub struct Split:
    rest: str
    sep: str
    done: bool

extend Split implements Iterator:
    type Item = str

    fn next(mut self) -> Option[str]:
        if self.done:
            return None
        rest = self.rest
        match rest.split_once(self.sep):
            Some((part, remaining)):
                self.rest = remaining
                return Some(part)
            None:
                self.done = true
                return Some(rest)

@derive(Copy)
pub struct SplitWhitespace:
    rest: str

extend SplitWhitespace implements Iterator:
    type Item = str

    fn next(mut self) -> Option[str]:
        rest = self.rest
        n = rest.len()
        start = 0
        while start < n:
            at = start
            if not is_split_space(rest.next_char(at)):
                break
            start = at
        if start >= n:
            self.rest = rest[n..]
            return None
        end = start
        while end < n:
            at = end
            if is_split_space(rest.next_char(at)):
                break
            end = at
        self.rest = rest[end..]
        return Some(rest[start..end])
