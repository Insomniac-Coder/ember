#$ test: run-pass
#$ rules: LT-48, LT-1
#$ stdin: m
#$ stdout: [9]
# `[LT-48]` (verdict 152) — `runtime` on `show`'s parameter checks its kind while running: given a
# `Mine`, chosen while running, whose `pick` gives back its own list, the check passes and `show`
# prints `[9]`. Without `runtime` the same program is refused when compiling (`E3021`).

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
