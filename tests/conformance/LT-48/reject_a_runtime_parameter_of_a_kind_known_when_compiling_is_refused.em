#$ test: compile-fail
#$ rules: LT-48, LT-1
#$ not-help: `runtime`
# `[LT-48]` (verdict 152) — where the kind is known when compiling, `runtime` changes nothing: `show`
# given a `Yours` is refused at `xs.push(owns 3)` when compiling, as without the word; the value whose kind
# is chosen while running is checked while running, and is no error.

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
    xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(r)

fn main():
    y = Yours(items=[9])
    show(ref y)
    p: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    show(p)
