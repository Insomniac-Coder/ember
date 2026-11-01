#$ test: compile-fail
#$ rules: LT-1, BRW-1
#$ help: to check it while running, write `runtime` on the parameter: `fn ping(p: runtime dyn Picker):`
# `[LT-1]` (verdict 151) — functions only each other call, and nothing else, are checked too: no kind
# reaches `ping`'s parameter, so its call points into every argument (the worst case) and the change of
# `xs` while `r` is used is refused.

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

fn ping(p: dyn Picker, n: int):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)
    pong(p, n)

fn pong(p: dyn Picker, n: int):
    if n > 0:
        ping(p, n - 1)

fn main():
    println("hi")
