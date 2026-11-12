#$ test: compile-fail
#$ rules: TYP-47, GRM-40
# `[TYP-47]` — a pack is the last of its type parameters, and its values' parameter is the
# function's last positional one: a parameter after it has a default and is passed by name only.
# Anywhere else either is `E0100`, and so are two packs and a value for the pack's values.

[Ts..., T]    #$ error[E0100]: a pack is the last of its type parameters
fn first(x: T, values: Ts...):
    pass

[Ts..., Us...]    #$ error[E0100]: a declaration takes one pack
fn two(values: Ts...):
    pass

[Ts...]
fn after(values: Ts..., n: int):    #$ error[E0100]: `n` follows `values`, a pack's values, so it is passed by name and needs a default
    pass

[Ts...]
fn defaulted(values: Ts... = 1):    #$ error[E0100]: `values` takes a pack's values, which have no default
    pass

[Ts...]
fn takes_one(values: Ts..., sep: StrView = " "):
    pass

fn main():
    takes_one(1, 2, sep = ", ")
