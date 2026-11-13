# Clause SPN-1.3: "the source is not changed through another name, handed over or ended while the view
# is used" -- once the last use of every view has passed, the source may change, whatever spelling
# made the view.
#$ test: run-pass
#$ rules: SPN-1, BRW-2, LT-5
#$ stdout: 1 1 1
#$ stdout: [9, 2, 3, 4]
#$ stdout: abc 2

fn head(xs: Span[int]) -> Span[int]:
    return xs[0..1]

fn main():
    xs = [1, 2, 3]
    a: Span[int] = xs
    b = xs.as_span()
    c = head(xs)
    println(a[0], b[0], c[0])
    xs[0] = 9
    xs.push(owns 4)
    println(xs)
    s = String("ab")
    w: StrView = s
    n = w.len()
    s.push_str("c")
    println(s, n)
