#$ test: run-pass
#$ rules: STR-5, TYP-39
#$ profiles: debug, development, release
#$ stdout: Secret(***)
#$ Secret(***) [Secret(***), Secret(***)] Some(Secret(***))
#$ Holder(secret=Secret(***), n=2)
# `[STR-5]` — a written `Debug` replaces the implicit field-wise one, wherever
# the value is formatted: alone, and as an element, a field or a payload of
# another value's text (`[TYP-39]`). A type with no `Display` prints its
# `Debug`.

from std.fmt import Formatter, FmtError

struct Secret implements Debug:
    key: int

    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("Secret(***)")

struct Holder:
    secret: Secret
    n: int

fn main():
    s = Secret(42)
    println(s)
    println(f"{s!r} {[Secret(1), Secret(2)]} {Some(Secret(3))}")
    println(Holder(Secret(4), 2))
