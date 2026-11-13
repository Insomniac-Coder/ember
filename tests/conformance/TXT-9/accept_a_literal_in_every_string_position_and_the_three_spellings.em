#$ test: run-pass
#$ rules: TXT-9
#$ stdout: a b c d e true
## `[TXT-9]` - a string literal produces a new `String` in an initialiser, an argument, a field, a
## collection element and a return value; `String(s)`, `s.to_string()` and `String.from(s)` mean the
## same for a view of your own.

struct N:
    name: String

fn take(owns s: String) -> String:
    return s

fn give() -> String:
    return "c"

fn main():
    a: String = "a"
    n = N("b")
    xs: Array[String] = ["d"]
    v = "e"
    f = String(v)
    g = v.to_string()
    h = String.from(v)
    println(take(owns a), n.name, give(), xs[0], f, g == h)
