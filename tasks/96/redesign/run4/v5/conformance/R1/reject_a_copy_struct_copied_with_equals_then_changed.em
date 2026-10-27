#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: for a copy of your own write `q = copy of p`; to change `p` write `q = ref mut p`
## design rules: R1
## R1: a `Copy` struct copied with `=` from a name that is still used afterwards is not then changed
## through the new name (E3023): the line says `copy of` (a copy of your own) or `ref mut` (the
## original), so no copy is changed by mistake (AN-44's struct case, which prints 1 silently today).

@derive(Copy)
struct Vec2:
    x: int
    y: int

fn main():
    p = Vec2(x=1, y=2)
    q = p
    q.x += 1                   #$ error[E3023]: `q.x += 1` changes `q`, a copy of `p` made by `q = p` (line 15), and `p` is used again on line 17
    println(p.x)
