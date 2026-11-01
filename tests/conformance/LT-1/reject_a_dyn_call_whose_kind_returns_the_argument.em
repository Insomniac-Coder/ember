#$ test: compile-fail
#$ rules: LT-1, LT-22, BRW-1
#$ not-help: `runtime`
# `[LT-1]` (verdict 151) — `show` is given a `Yours`, whose `pick` gives back the list it was handed:
# `r` points into `xs`, which changes while `r` is used, so `show` is refused at the change, as the
# kind is known when compiling. `keep`, the same code given a `Mine`, compiles. (The spec's spelling,
# `show(Yours(items=[9]))`, is not accepted by this compiler yet.)

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

fn keep(p: dyn Picker):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)
    println(r)

fn show(p: dyn Picker):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn main():
    m = Mine(items=[9])
    keep(ref m)
    y = Yours(items=[9])
    show(ref y)
