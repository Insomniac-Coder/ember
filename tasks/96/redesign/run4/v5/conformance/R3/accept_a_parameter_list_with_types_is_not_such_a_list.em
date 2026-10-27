#$ test: run-pass
#$ rules: R3, R12
#$ stdout: 5
## R3, R12: the nearest programs that are allowed: each untyped parameter says its word, or the
## parameters have types, which is not such a list (a parameter's word is on its own name, R4).

class Ui:
    x: int = 0

fn main():
    u = Ui()
    nudge: fn(ref mut Ui, int) = fn(ref mut ui, ref dx): ui.x += dx
    nudge(ref mut u, 2)
    push: fn(ref mut Ui, int) = fn(ref mut ui: Ui, dx: int): ui.x += dx
    push(ref mut u, 3)
    println(u.x)
