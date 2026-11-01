#$ test: compile-fail
#$ rules: LT-48, LT-1
#$ not-help: `runtime`
# `[LT-48]` (verdict 152) — `runtime` checks its parameter's value: `q` may be `p` or `o`, so a check of
# `p` would not tell what `q` is, and the change of `xs` that a `Yours` makes unsafe is refused when
# compiling.

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
    fn show(p: runtime dyn Picker, o: dyn Picker):
        q = p if input() == "p" else o
        xs = [1, 2]
        r = q.pick(xs)
        xs.push(owns 3)    #$ error[E3021]: `xs` changes while `r` may still look at it: `q` may be a `Yours`, and which kind it is is only known while running
        println(r)

fn main():
    s = Shower(n=1)
    m = Mine(items=[9])
    p: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    s.show(p, ref m)
