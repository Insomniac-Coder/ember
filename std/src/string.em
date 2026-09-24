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
