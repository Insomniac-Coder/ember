#$ test: compile-fail
#$ rules: LT-1, LT-48, BRW-1
#$ help: to check it while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`
# `[LT-1]`, `[LT-48]` (verdict 151) — a `dyn` value taken from a list is not followed to where it was
# made: `show` may be given any kind, so its call points into every argument (the worst case) and the
# change of `xs` while `r` is used is refused, with the help to check it while running.

interface Picker:
    fn pick(other: Array[int]) -> Span[int]

struct Mine implements Picker:
    items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return this.items

fn show(p: dyn Picker):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn main():
    ps: Array[owns dyn Picker] = [owns Mine(items=[9])]
    show(ps[0])
