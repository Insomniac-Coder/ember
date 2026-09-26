#$ test: run-pass
#$ rules: LT-22, LT-3
#$ stdout: x y
#$ stdout: ann 2
# `[LT-22]` (D-357) — a function whose result has a field-by-field summary
# may be given literals, whose region is `static`: there is nothing for the
# result to retain. The compiler's own check of the summary at the call
# panicked ("names no source region slot") on every such call.

fn pair(a: str, b: str) -> (str, str):
    return (a, b)

fn main():
    (p, q) = pair("x", "y")
    println(p, q)
    s = String.from("ann")
    (r, t) = pair(s.as_str(), "2")
    println(r, t)
