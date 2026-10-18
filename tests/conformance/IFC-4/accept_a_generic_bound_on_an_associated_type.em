#$ test: run-pass
#$ rules: IFC-4, TYP-17, GRM-8c
#$ stdout: 3.0
# `[IFC-4]` (D-674) — an associated type bounded by a generic interface named without positional
# arguments (`type Scalar: Add[Output = Scalar]`) takes its defaults with `Self` the associated
# type, as a parameter's bound does: `Add[Self.Scalar]`. It was recorded as the bare interface, which
# `f64` implemented nowhere (E2040 at the `extend`) and which gave a generic body no `+` (E2040).

interface Shape:
    type Scalar: Add[Output = Scalar]
    fn width(self) -> Scalar

struct Box2:
    w: f64

extend Box2 implements Shape:
    type Scalar = f64
    fn width(self) -> f64:
        return self.w

fn twice[S: Shape](s: S) -> S.Scalar:
    return s.width() + s.width()

fn main():
    println(twice(Box2(w=1.5)))
