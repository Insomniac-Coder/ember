#$ test: compile-fail
#$ rules: TYP-35, TYP-16
## `[TYP-35]` — `Length[Feet]` is not `Length[Meters]`, though no field says so.

struct Meters:
    pass

struct Feet:
    pass

struct Length[Unit]:
    value: int

fn take(a: Length[Meters]) -> int:
    return a.value

fn main():
    println(take(Length[Feet](4)))    #$ error[E2020]
