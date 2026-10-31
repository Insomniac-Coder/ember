## `std.string` — Part XV's text module. `String` and `str` are compiler-known
## until the library can declare them; this module holds what can already be
## written.

from std.fmt import Formatter, FmtError

## `[TXT-10]` (ODR-029) — why `s.parse[T]()` failed: the text was empty, held
## something that is not part of a `T` literal, or named a value outside `T`.
## A unit-only enum, so it is `Copy`, `Eq` and `Debug`.
pub enum ParseError:
    Empty
    Invalid
    Overflow

## `[ERR-3]`, `[ERR-8]` (D-764) — an error type: `?` carries a failed `parse` into an `AnyError`
## (`s.parse[int]()?` in a function returning `Result[T]`), and its text says why.
extend ParseError implements Display:
    ## It allocates as the `String` it writes to grows.
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return match this:
            Empty => f.write_str("the text is empty")
            Invalid => f.write_str("the text is not a value of this type")
            Overflow => f.write_str("the value is outside this type's range")

extend ParseError implements Error:
    pass

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
extend StrView:
    pub fn chars() -> Chars:
        return Chars(text = this, at = 0)

    pub fn char_indices() -> CharIndices:
        return CharIndices(text = this, at = 0)

    pub fn bytes() -> Bytes:
        return Bytes(text = this, at = 0)

    pub fn lines() -> Lines:
        return Lines(rest = this)

    pub fn split(sep: StrView) -> Split:
        if sep.len() == 0:
            panic("split: the separator is empty")
        return Split(rest = this, sep = sep, done = false)

    pub fn split_whitespace() -> SplitWhitespace:
        return SplitWhitespace(rest = this)

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

## `[TXT-10]` (ODR-183) — a character's class, as Python's `isalpha`, `isdigit`, `isalnum` and
## `isspace` of one character give it (Unicode; no locale), and the same restricted to ASCII. An
## ASCII character is decided here; the tables past it are the runtime's (`tools/unicode_case.py`).
extend char:
    ## A letter: Unicode's categories Lu, Ll, Lt, Lm and Lo (Python's `isalpha`).
    pub fn is_alphabetic() -> bool:
        code = this as u32
        if code < 0x80:
            return (code | 0x20) >= 0x61 and (code | 0x20) <= 0x7A
        return char_class(this, 0)

    ## A digit of any script, superscripts included (Python's `isdigit`): `'7'`, `'٣'`, `'²'`.
    pub fn is_digit() -> bool:
        code = this as u32
        if code < 0x80:
            return code >= 0x30 and code <= 0x39
        return char_class(this, 1)

    ## A letter or a number of any kind (Python's `isalnum`): alphabetic, or numeric (`'½'`).
    pub fn is_alphanumeric() -> bool:
        code = this as u32
        if code < 0x80:
            return (code >= 0x30 and code <= 0x39) or ((code | 0x20) >= 0x61 and (code | 0x20) <= 0x7A)
        return char_class(this, 0) or char_class(this, 2)

    ## White space, as `split_whitespace` splits at (Python's `isspace`).
    pub fn is_whitespace() -> bool:
        return is_split_space(this)

    pub fn is_ascii_alphabetic() -> bool:
        return (this as u32) < 0x80 and this.is_alphabetic()

    pub fn is_ascii_digit() -> bool:
        return (this as u32) < 0x80 and this.is_digit()

    pub fn is_ascii_alphanumeric() -> bool:
        return (this as u32) < 0x80 and this.is_alphanumeric()

    pub fn is_ascii_whitespace() -> bool:
        return (this as u32) < 0x80 and is_split_space(this)

## The runtime's own (`ember_hir::RuntimeFn`): whether `c`, past ASCII, is alphabetic (`which` 0),
## a digit (1) or numeric (2). This staging body never runs.
fn char_class(c: char, which: int) -> bool:
    return false

@derive(Copy)
pub struct Chars:
    text: StrView
    at: int

extend Chars implements Iterator:
    type Item = char

    ## `next_char` decodes the character at `at` and moves `at` past it;
    ## `at` stays on a boundary.
    fn next() -> Option[char]:
        if this.at >= this.text.len():
            return None
        return Some(this.text.next_char(this.at))

@derive(Copy)
pub struct CharIndices:
    text: StrView
    at: int

extend CharIndices implements Iterator:
    type Item = (int, char)

    fn next() -> Option[(int, char)]:
        if this.at >= this.text.len():
            return None
        here = this.at
        return Some((here, this.text.next_char(this.at)))

@derive(Copy)
pub struct Bytes:
    text: StrView
    at: int

extend Bytes implements Iterator:
    type Item = u8

    fn next() -> Option[u8]:
        if this.at >= this.text.len():
            return None
        b = this.text.as_bytes()[this.at]
        this.at += 1
        return Some(b)

@derive(Copy)
pub struct Lines:
    rest: StrView

extend Lines implements Iterator:
    type Item = StrView

    ## The text is copied out of the field first, so a line borrows the
    ## text, not this iterator.
    fn next() -> Option[StrView] from rest:
        rest = this.rest
        if rest.len() == 0:
            return None
        match rest.find("\n"):
            Some(at):
                line = rest[..at]
                this.rest = rest[at + 1..]
                if line.ends_with("\r"):
                    return Some(line[..line.len() - 1])
                return Some(line)
            None:
                this.rest = rest[rest.len()..]
                return Some(rest)

@derive(Copy)
pub struct Split:
    rest: StrView
    sep: StrView
    done: bool

extend Split implements Iterator:
    type Item = StrView

    fn next() -> Option[StrView] from rest:
        if this.done:
            return None
        rest = this.rest
        match rest.split_once(this.sep):
            Some((part, remaining)):
                this.rest = remaining
                return Some(part)
            None:
                this.done = true
                return Some(rest)

@derive(Copy)
pub struct SplitWhitespace:
    rest: StrView

extend SplitWhitespace implements Iterator:
    type Item = StrView

    fn next() -> Option[StrView] from rest:
        rest = this.rest
        n = rest.len()
        start = 0
        while start < n:
            at = start
            if not is_split_space(rest.next_char(at)):
                break
            start = at
        if start >= n:
            this.rest = rest[n..]
            return None
        end = start
        while end < n:
            at = end
            if is_split_space(rest.next_char(at)):
                break
            end = at
        this.rest = rest[end..]
        return Some(rest[start..end])
