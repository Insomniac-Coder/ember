#$ test: compile-fail
#$ rules: TYP-25
## `[TYP-25]` - for a positional argument after a named one, the line to write names it with the first
## parameter that no other argument of the call gives, before it or after it, so that the line gives each
## parameter once; a memberwise constructor is such a call (`[STR-1]`).

fn area(w: int, h: int, scale: int = 1) -> int:
    return w * h * scale

struct Rect:
    w: int
    h: int

fn main():
    c = area(w=2, 3, h=4)            #$ error[E2021]: `area(w=2, 3, h=4)` gives `3` after the named `w=2`, and positional arguments come first; write `area(w=2, scale=3, h=4)`
    g = Rect(w=2, 3)                 #$ error[E2021]: `Rect(w=2, 3)` gives `3` after the named `w=2`, and positional arguments come first; write `Rect(w=2, h=3)`
    d = Rect(3, w=2)                 #$ error[E2021]: `Rect(3, w=2)` gives `w` twice
    println(c, g.w, d.w)
