#$ test: compile-fail
#$ rules: LT-1, LT-48, BRW-1
#$ help: to check it while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`
# `[LT-1]`, `[LT-48]` (verdicts 151, 152) — which kind reaches `show` is known only while running: `p`
# may be a `Mine` or a `Yours`, and a `Yours` makes the change of `xs` unsafe while `r` is used. The
# message names the kind, and the help says how to check it while running. (The spec's spelling,
# `p: dyn Picker = Mine(items=[9]) if input() == "m" else Yours(items=[9])`, is not accepted by this
# compiler yet.)

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
    xs.push(owns 3)    #$ error[E3021]: `xs` changes while `r` may still look at it: `p` may be a `Yours`, and which kind it is is only known while running
    println(r)

fn main():
    p: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    show(p)
