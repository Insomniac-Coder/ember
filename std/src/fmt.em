## `std.fmt` — what a written `Display` writes into (Part XV, `[STD-18]`).
##
## `std.core` declares `Display` (Part IV §8): `fn fmt(self, mut f:
## Formatter) -> Result[void, FmtError]`. A type that implements it writes its
## text through the `Formatter` it is given, and every place the language
## turns a value into text (`f"{x}"`, `print(x)`, `main`'s error) uses that
## text. The numbers, `bool`, `char`, text, the collections, tuples,
## `Option`, `Result`, `Box` and `Cell` have their `Display` from Part IV
## §8's table, written by the compiler (`[TYP-36]`, `[TYP-39]`); a struct, an
## enum or a class has one only by implementing `Display` (ADR-186).

## The text being built, borrowed for one `fmt` call: what `f"{x}"` or
## `print(x)` is writing at the moment it reaches `x`. Writing appends.
@view
pub struct Formatter:
    out: ref mut String

## Why a `Formatter` refused text. Text built in a `String` takes everything,
## so nothing here produces one; a `fmt` hands one on from a `write_str` it
## called (`return f.write_str(…)`, or `?`).
pub enum FmtError:
    Failed

extend Formatter:
    ## Appends `s` to the text being built.
    pub fn write_str(mut self, s: str) -> Result[void, FmtError]:
        self.out.push_str(s)
        return Ok(())

## `[LEX-19]`, `[STD-9]` — how the compiler writes a value whose type has a
## written `Display` into text it is building (an f-string's, or `print`'s
## argument): the value's `fmt`, given a `Formatter` over that text. Nothing
## is allocated beyond what `fmt` itself writes.
fn write_display[T: Display](value: T, mut out: String):
    f = Formatter(ref mut out)
    _ = value.fmt(f)

## `[STR-5]` — the same for a written `Debug`, which the compiler calls
## wherever the value is formatted, alone or inside another value.
fn write_debug[T: Debug](value: T, mut out: String):
    f = Formatter(ref mut out)
    _ = value.fmt_debug(f)

## `[STR-5]` — `value`'s `Debug` text appended to `out`, as `f"{value!r}"`
## writes it. Compiler-known.
fn append_debug[T: Debug](value: T, out: ref mut String):
    pass

## `[STR-5]` — a type's own field-wise `Debug` as its `fmt_debug`, for the
## table of a `dyn` interface needing `Debug` (`dyn Error`) when the type
## writes none, and for `@derive(Display)` on a struct (`[DRV-1]`). It writes
## into the `Formatter`'s text directly (ADR-191).
pub fn implicit_fmt_debug[T: Debug](value: T, mut f: Formatter) -> Result[void, FmtError]:
    append_debug(value, f.out)
    return Ok(())
