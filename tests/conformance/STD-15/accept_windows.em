#$ test: run-pass
#$ rules: STD-15, SPN-1, SPN-4
#$ stdout:
#$ 1-3 2-4 3-5
#$ 0 1 4
#$ 4
#$ ab bc
# `[STD-15]` (ODR-031) — `windows(n)` yields every run of `n` neighbours, one
# step apart, as shared views: `len - n + 1` of them, none when `n > len`. It
# works on an `Array`, through `next`, and on a view.

fn main():
    xs = [1, 2, 3, 4, 5]
    shown: Array[String] = []
    for w in xs.windows(3):
        shown.push(f"{w[0]}-{w[w.len() - 1]}")
    println(shown.join(" "))

    counts: Array[int] = []
    for n in [9, 5, 2]:
        count = 0
        for _w in xs.windows(n):
            count += 1
        counts.push(count)
    println(counts[0], counts[1], counts[2])

    it = xs.windows(4)
    match it.next():
        Some(w): println(w[3])
        None: println("none")

    names = [String.from("a"), String.from("b"), String.from("c")]
    pairs: Array[String] = []
    for w in names.as_span().windows(2):
        pairs.push(f"{w[0]}{w[1]}")
    println(pairs.join(" "))
