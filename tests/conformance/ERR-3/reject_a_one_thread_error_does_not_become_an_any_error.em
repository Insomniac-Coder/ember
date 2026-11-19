#$ test: compile-fail
#$ rules: ERR-3, ERR-8, ERR-1
# `[ERR-3]` — an error holding a value that stays on one thread (a `shares` owner, a `weakref` link, a
# `Cell`) is an `Error` on its thread, but `AnyError`'s `From[E]` needs `E: needs (Error, Send,
# ThreadShared)` (`[ERR-8]`, `[ERR-1]`), so `?` and `.into()` into `AnyError` are `E2040` at that line,
# naming the field.

from std.fmt import Formatter, FmtError

class Script:
    n: int

struct Owner:
    script: shares Script

struct Counting:
    seen: Cell[int]

extend Owner implements Display, Error:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("owner")

extend Counting implements Display, Error:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("counting")

fn owner() -> Result[int, Owner]:
    return Ok(1)

fn counting() -> Result[int, Counting]:
    return Ok(2)

fn through_question() -> Result[int]:
    a = owner()?    #$ error[E2040]: `Owner` cannot become an `AnyError`, which any thread may hold: `Owner.script` is a `shares Script` owner, which stays on one thread, so it may hold what one thread alone may use
    return Ok(a)

fn through_cell() -> Result[int]:
    b = counting()?    #$ error[E2040]: `Counting` cannot become an `AnyError`, which any thread may hold: `Counting.seen` is a `Cell[i64]`, which several threads may not read at once, so it may hold what one thread alone may use
    return Ok(b)

fn main():
    println(1)
