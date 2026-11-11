#$ test: run-fail
#$ rules: LT-48, LT-1
#$ stdin: y
#$ panics: `xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed
# `[LT-48]` (verdict 152) — `runtime` on `show`'s parameter checks its kind while running, at the line a
# kind makes unsafe: given a `Yours`, chosen while running, the program stops at `xs.push(owns 3)`. (The
# spec's spelling, `p: dyn Picker = Mine(items=[9]) if input() == "m" else Yours(items=[9])`, is not
# accepted by this compiler yet.)

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

fn show(p: runtime dyn Picker):
    xs = [1, 2]
    r = p.pick(xs)
    xs.push(owns 3)
    println(r)

fn main():
    p: owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])
    show(p)
