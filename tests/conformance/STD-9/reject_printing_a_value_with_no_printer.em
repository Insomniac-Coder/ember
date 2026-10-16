#$ test: compile-fail
#$ rules: STD-9, TYP-39
#$ error[E2040]: `RefCell[i64]` implements neither `Display` nor `Debug`, which printing it needs
#$ error[E2040]: `RefCell[i64]` does not implement `Debug`, which printing a `Holder` needs
# `[STD-9]` — an argument's text is its `Display`, else its `Debug`; a value with neither is
# `E2040`, whose help is to implement `Display`. A `RefCell` has neither (a `Cell` prints as what
# it holds, ODR-094), and a struct's implicit `Debug` (`[STR-5]`) needs every field to have one.

struct Holder:
    c: RefCell[int]

fn main():
    println(RefCell(1))
    h = Holder(c=RefCell(1))
    println(h)
