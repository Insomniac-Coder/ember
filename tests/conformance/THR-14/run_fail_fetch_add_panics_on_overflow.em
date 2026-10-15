#$ test: run-fail
#$ rules: THR-14, TYP-8
#$ panics: integer overflow in `+`
#$ stdout: None 120
# `[THR-14]` — `fetch_add` panics when the result overflows, as `+` does (`[TYP-8]`);
# `checked_fetch_add` gives `None` and stores nothing.

from std.sync import Atomic

fn main():
    a: Atomic[i8] = Atomic(120)
    println(a.checked_fetch_add(10), a.load())
    a.fetch_add(10)
