#$ test: run-pass
#$ rules: ERR-3, ERR-8
#$ stdout: owner 7
#$ stdout: load failed: owner 7
#$ stdout: ok 3
# `[ERR-3]` — an error holding a value that stays on one thread is an `Error` on its thread as any
# other: matched and printed by its `Display`, returned in a `Result[T, LoadError]`, and passed through
# `?` into another `Result` whose error type takes it. An error any thread may hold goes through `?`
# into `AnyError` (`[ERR-8]`).

from std.fmt import Formatter, FmtError

class Script:
    n: int

struct Owner:
    script: shares Script

extend Owner implements Display, Error:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"owner {this.script.n}")

struct Outer:
    inner: Owner

extend Outer implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"load failed: {this.inner}")

extend Outer implements From[Owner]:
    static fn from(owns value: Owner) -> Outer:
        return Outer(value)

fn inner() -> Result[int, Owner]:
    return Err(Owner(shares Script(7)))

fn outer() -> Result[int, Outer]:
    v = inner()?
    return Ok(v)

struct Plain:
    code: int

extend Plain implements Display, Error:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("plain")

fn fine() -> Result[int]:
    n = three()?
    return Ok(n)

fn three() -> Result[int, Plain]:
    return Ok(3)

fn main():
    match inner():
        Ok(v) => println(v)
        Err(e) => println(e)
    match outer():
        Ok(v) => println(v)
        Err(e) => println(e)
    match fine():
        Ok(v) => println(f"ok {v}")
        Err(e) => println(e)
