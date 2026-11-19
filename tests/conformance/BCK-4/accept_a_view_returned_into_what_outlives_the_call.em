# Clause BCK-4.2: "a view, or a value holding views, kept or returned where it would outlive what it
# points into (...) is `E3063`" -- a view of what the caller gave, or of text written in the program,
# outlives nothing it points into.
#$ test: run-pass
#$ rules: BCK-4, LT-1, LT-3, LT-47
#$ stdout: a lit
#$ stdout: b

fn first(s: StrView) -> StrView:
    return s[0..1]

fn literal() -> StrView:
    return "lit"

fn put(ref mut out: StrView, v: StrView):
    out = v[1..2]

fn main():
    text = String("ab")
    println(first(text), literal())
    name: StrView = "x"
    put(ref mut name, text)
    println(name)
