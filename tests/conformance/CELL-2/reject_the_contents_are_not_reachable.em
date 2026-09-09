#$ test: compile-fail
#$ rules: CELL-2, MOD-2
# "`Cell` never hands out a reference to its contents, so no aliasing rule can
# be violated and **no runtime check is needed**."
#
# That sentence is what makes the whole type sound: the borrow checker is not
# weakened for `Cell`, and it does not have to be, because nothing escapes.
# What holds it up is `[MOD-2]`'s ordinary privacy — the payload is a private
# field of a struct whose declaring module is one no program can be in — so all
# three routes to it are refused, including the one that matters most, taking a
# `ref`.
#
# The field is named `value`, an ordinary identifier, and that is deliberate.
# An unspellable name (`$value`) was tried first and broke `[CG-C-1]`: the
# backend writes field names into the C verbatim and `$` in an identifier is a
# compiler extension, which the rule forbids relying on. Privacy is the
# mechanism; the name is just a name.

fn main():
    c: Cell[i32] = Cell(1)
    println(c.value)          #$ error[E1020]: `value` is private to `Cell_i32`'s module
    c.value = 9               #$ error[E1020]: `value` is private to `Cell_i32`'s module
    r = ref c.value           #$ error[E1020]: `value` is private to `Cell_i32`'s module
    println(r)
