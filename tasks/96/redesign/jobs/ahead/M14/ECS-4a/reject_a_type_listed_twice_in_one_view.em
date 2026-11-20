# Clause ECS-4a.1: "The component types listed in one `view`, `view_mut`, `all_of`, `any_of`, `group`
# or `exclude` are distinct, judged after generic parameters are filled in (`[TYP-17]`, at each
# use): a type listed twice, or listed and excluded in one view, is `E2074`, naming the type and the
# line to write"; "A generic function whose list becomes a repeat for one use is refused at that use,
# naming it ("in `step` used with `A = B = Position`")."
#$ test: compile-fail
#$ rules: ECS-4a, ECS-4, TYP-17
# The spec's message, with this file's line number; listed and excluded is code only; the generic
# use claims the part the spec gives.

from std.ecs import Registry

@derive(Copy)
struct Position:
    x: int

[A, B]
fn step(ref mut reg: Registry):
    for e, ref mut a, ref mut b in reg.view_mut[A, B]():
        pass

fn main():
    reg = Registry()
    for e, ref mut p, ref mut q in reg.view_mut[Position, Position]():    #$ error[E2074]: `view_mut[Position, Position]` (line 24) names `Position` twice; a view gives each component of an entity once, so two of its items would name one place: write `view_mut[Position]`
        pass
    for e, p in reg.view[Position]().exclude[Position]():    #$ error[E2074]
        pass
    step[Position, Position](ref mut reg)    #$ error[E2074]: in `step` used with `A = B = Position`
