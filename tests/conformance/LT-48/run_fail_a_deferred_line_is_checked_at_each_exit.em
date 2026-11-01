#$ test: run-fail
#$ rules: LT-48, CTL-8
#$ stdin: y
#$ stdin: late
#$ stdout: end
#$ panics: `xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed
# `[LT-48]` (verdict 152) — a `defer`'s body runs at each exit, and each place it runs is checked: given
# a `Yours`, `show` stops at the deferred `xs.push(owns 3)` when it ends at its last line as when it
# returns early.

interface Picker:
    fn pick(other: Array[int]) -> Span[int]

struct Mine implements Picker:
    items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return this.items

struct Yours implements Picker:
    items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return other

struct Shower:
    n: int
    fn show(p: runtime dyn Picker):
        xs = [1, 2]
        r = p.pick(xs)
        defer:
            xs.push(owns 3)
            println(r)
        if input() == "early":
            return
        println("end")

fn main():
    s = Shower(n=1)
    p: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    s.show(p)
