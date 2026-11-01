#$ test: run-pass
#$ rules: GRM-41, FN-1
#$ stdout: 3 hi
#$ stdout: ab
## `[GRM-41]` — a function's first line gives its parameters, each with its word before its name
## (`ref mut`, `owns`, or none, a read-only view), then `->` and the result type, then `from` and
## what the result points into.

fn bump(ref mut n: int, owns s: String) -> String:
    n += 1
    return s

fn longer(a: StrView, b: StrView) -> StrView from a:
    if a.len() >= b.len():
        return a
    return a

fn main():
    k = 2
    s = bump(ref mut k, owns String("hi"))
    println(k, s)
    println(longer("ab", "c"))
