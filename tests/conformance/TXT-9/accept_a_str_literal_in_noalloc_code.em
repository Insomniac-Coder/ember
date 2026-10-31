#$ test: run-pass
#$ rules: TXT-9
#$ stdout: 4
# `[TXT-9]` — where a `str` is expected a literal is a static `str` and allocates nothing.

@noalloc
fn named() -> int:
    s: StrView = "text"
    return s.len()

fn main():
    println(named())
