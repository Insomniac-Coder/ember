#$ test: run-pass
#$ rules: TYP-35, TYP-16
#$ stdout: 3 4
## `[TYP-35]` — a type parameter no field names (a phantom) is legal, and two
## instances that differ only in it are two types.

struct Meters:
    pass

struct Feet:
    pass

struct Length[Unit]:
    value: int

fn add_meters(a: Length[Meters], b: Length[Meters]) -> Length[Meters]:
    return Length[Meters](a.value + b.value)

fn main():
    m = add_meters(Length[Meters](1), Length[Meters](2))
    f = Length[Feet](4)
    println(m.value, f.value)
