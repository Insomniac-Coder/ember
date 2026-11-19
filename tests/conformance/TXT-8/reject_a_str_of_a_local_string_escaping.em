#$ test: compile-fail
#$ rules: TXT-8
#$ error[E3063]: this points into `s`
#$ error[E3063]: this points into `t`
# `[TXT-8]` — a `str` is a view and carries a region, like every `Span`: a `str` of a local
# `String` cannot leave the function, by itself or inside a struct that holds it.

struct Holder:
    view: StrView

fn bad() -> StrView:
    s: String = "abc"
    return s.as_str()

fn make() -> Holder:
    t: String = "def"
    return Holder(t.as_str())

fn main():
    println(bad(), make().view)
