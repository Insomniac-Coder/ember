#$ test: compile-fail
#$ rules: R3, R12
#$ help: say what `dx` is: `fn(ref mut ui, ref dx)`, or give both their types: `fn(ref mut ui: Ui, dx: int)`
## R3, R12: the same rule for a lambda's parameters written without types (E0116; `V3/rules.md`
## section 3's message).

class Ui:
    x: int = 0

fn main():
    u = Ui()
    nudge: fn(ref mut Ui, int) = fn(ref mut ui, dx): ui.x += dx    #$ error[E0116]: `ref mut` here belongs to `ui` only
    nudge(ref mut u, 2)
