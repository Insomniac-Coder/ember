#$ test: run-pass
#$ rules: LT-1, LT-22, TYP-22
#$ stdout: [9]
#$ stdout: [9]
#$ stdout: [4]
# `[LT-1]` (verdict 151) — a `dyn` method's call is decided by the code of each kind that may reach it:
# `show` is given a `Mine`, whose `pick` gives back its own list, so `r` points into `p` only and `xs`
# may change while `r` is used. The kind is followed through the call to `outer` too. (The spec's
# spelling, `show(Mine(items=[9]))`, is not accepted by this compiler yet.)

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

fn show(p: dyn Picker):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)
    println(r)

fn outer(p: dyn Picker):
    show(p)

fn main():
    m = Mine(items=[9])
    show(ref m)
    outer(ref m)
    y = Yours(items=[5])
    println(y.pick([4]))
