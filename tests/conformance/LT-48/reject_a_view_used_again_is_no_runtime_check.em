#$ test: compile-fail
#$ rules: LT-48, LT-1, BRW-1
# `[LT-48]` (verdict 152) — only a line the call's result makes unsafe is checked while running: `v`
# views `xs` and is used after `xs` changes, whatever kind `p` is, so the change is refused when
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
    fn show(p: runtime dyn Picker):
        xs = [1, 2]
        v = ref xs
        r = p.pick(v)
        xs.push(owns 3)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
        println(r, v)

fn main():
    s = Shower(n=1)
    p: Box[dyn Picker] = Box(Mine(items=[9])) if input() == "m" else Box(Yours(items=[9]))
    s.show(p)
