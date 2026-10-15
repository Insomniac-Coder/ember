#$ test: compile-fail
#$ rules: TXT-8
#$ error[E3060]: `s` does not live long enough
#$ error[E3060]: `t` does not live long enough
# `[TXT-8]` — a `str` is a view and carries a region, like every `Span`: a `str` of a local
# `String` cannot leave the function, by itself or inside a struct that holds it.

struct Holder:
    view: str

fn bad() -> str:
    s: String = "abc"
    return s.as_str()

fn make() -> Holder:
    t: String = "def"
    return Holder(t.as_str())

fn main():
    println(bad(), make().view)
