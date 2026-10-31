#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# `[TYP-17]` (D-676) — a `where` clause on an associated type is met at each call, and one that a
# bound's binding already fixes is checked against that type.

struct Opaque:
    x: int

[I: needs Iterator]
fn show_all(owns it: I) where I.Item: needs Display:
    for x in it:
        println(f"{x}")

[I: needs Iterator[Item = Opaque]]
fn fixed(owns it: I) where I.Item: needs Display:   #$ error[E2040]: `I.Item` is `Opaque`, which does not implement `Display`
    pass

fn main():
    show_all([Opaque(1)].into_iter())   #$ error[E2040]: `Opaque` does not implement `Display`, which `I.Item` requires
