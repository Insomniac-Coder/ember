#$ test: compile-fail
#$ rules: IFC-3, TYP-17
# `[IFC-3]` (D-532) — what an associated type's bound brings is its parents and nothing more:
# `Thing: Named` is a `Shape`, not a `Colored`.

interface Shape:
    fn area(self) -> int

interface Colored:
    fn color(self) -> int

interface Named: Shape:
    fn name(self) -> str

interface Holder:
    type Thing: Named
    fn get(self) -> Thing

fn tint[H: Holder](h: H) -> int:
    return h.get().color() #$ error[E2040]: `H.Thing` has no method `color`; its bounds do not provide one

fn main():
    pass
