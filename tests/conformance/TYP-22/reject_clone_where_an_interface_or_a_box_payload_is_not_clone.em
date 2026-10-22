#$ test: compile-fail
#$ rules: TYP-22, TYP-36, STR-5
## `[TYP-22]`, IX.1 (ODR-237) — an interface that requires `Clone` makes every implementing type
## `Clone`: one that is not is refused, by name. A `Box[dyn I]` whose interface does not require
## `Clone` is not `Clone`, nor is a `Box[T]` whose `T` is not, so a struct asking to derive it over
## such a field is refused too.

interface Shape: Clone:
    fn area(self) -> float

interface Plain:
    fn area(self) -> float

@no_derive(Clone)
struct Unique:
    id: int

extend Unique implements Shape:  #$ error[E2040]: `Shape` requires `Clone`, which `Unique` does not implement
    fn area(self) -> float:
        return 0.0

extend Unique implements Plain:
    fn area(self) -> float:
        return 1.0

@derive(Clone)
struct Holder:
    unique: Box[Unique]  #$ error[E2040]: field `unique` has type `Box[Unique]`, which does not implement `Clone`

fn main():
    plain: Box[dyn Plain] = Box(Unique(1))
    _copy = plain.clone()  #$ error[E1010]: `Box[dyn Plain]` has no method named `clone`
