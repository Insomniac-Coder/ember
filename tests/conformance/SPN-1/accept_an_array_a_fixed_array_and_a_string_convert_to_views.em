# Clause SPN-1.1: "An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]` or `StrView` where
# one is expected"
#$ test: run-pass
#$ rules: SPN-1, TYP-5, TYP-15
#$ stdout: 6 6 6
#$ stdout: ember 5
#$ stdout: 3 3
# A parameter, a typed local and a field whose type is a `Span` or a `StrView` each take the list, the
# fixed array or the string itself, with nothing written and nothing copied.

struct Window:
    rows: Span[int]

fn total(xs: Span[int]) -> int:
    t = 0
    for x in xs:
        t += x
    return t

fn size(text: StrView) -> int:
    return text.len()

fn main():
    xs = [1, 2, 3]
    fixed: [int; 3] = [3, 2, 1]
    s = String("ember")
    v: Span[int] = xs
    w: StrView = s
    win = Window(rows=fixed)
    println(total(xs), total(fixed), total(v))
    println(w, size(s))
    println(win.rows[0], win.rows.len())
