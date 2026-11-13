#$ test: compile-fail
#$ rules: TYP-47
# `[TYP-47]` — `...` is written after a pack's name only (`Cs...`, `values...`): a type formed over
# each type of a pack (`Option[Cs]...`, C++'s pattern expansion) is not Ember and is `E0100`, as is
# a pack's name standing for one of its types, `...` after a name that is no pack, and a pack
# spelled out among the arguments of a type that takes none. A pack's values are used one at a
# time; `values...` passes them on to a function over a pack only.

[Cs...]
fn options(values: Option[Cs]...):    #$ error[E0100]: `Option[Cs]...` forms a type over each type of a pack
    pass

[Cs...]
fn bare(values: Cs...) -> Cs:    #$ error[E0100]: `Cs` is a pack: its types are spelled out as `Cs...`
    pass

[T]
fn not_a_pack(x: (int, T...)):    #$ error[E0100]: `T...` spells a pack's types out, and `T` is one type
    pass

fn no_pack(x: (int, int...)):    #$ error[E0100]: `int...` spells a pack's types out, and `int` is no pack
    pass

[Cs...]
fn into_option(values: Cs...) -> Option[Cs...]:    #$ error[E0100]: `Option` takes no pack
    pass

[Ts...]
fn as_value(values: Ts...):
    held = values    #$ error[E2020]: `values` holds a pack's values, each of its own type, so it is no one value

fn two(a: int, b: int):
    pass

[Ts...]
fn spread_into_two(values: Ts...):
    two(values...)    #$ error[E2020]: `values...` passes a pack's values on, to a function whose last positional parameter takes a pack's values

fn main():
    pass
