#$ test: parse-pass
#$ rules: TYP-47, GRM-40, LEX-21
# `[TYP-47]` (verdict 116, C++'s spelling) — `...` after a type parameter's name makes it a pack
# (`[Ts...: needs Display]`), after a parameter's type takes the pack's values (`values: Ts...`),
# and spells the pack out among type arguments (`View[Cs...]`), a tuple's elements (`(Entity,
# Cs...)`), a function type's parameters (`fn(Ts...)`) and a call's arguments (`log(values...)`).
# `...` is one token, before `..` (`[LEX-21]`).

[Ts...: needs Display]
fn log(values: Ts..., sep: StrView = " "):
    pass

[Cs...]
struct View:
    rows: Array[(int, Cs...)]
    each: Array[fn(Cs...)]

[Cs...]
fn view() -> View[Cs...]:
    pass

[Ts...: needs Display]
fn shout(values: Ts...):
    log("!", values..., sep = "")
