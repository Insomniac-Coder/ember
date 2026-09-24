#$ test: compile-fail
#$ rules: STD-9
#$ error[E0900]: printing a `Holder` is not implemented yet
# D-197 — `println` of one value is checked for a printer like several are;
# a value with none used to reach the C compiler as a string. A struct's
# implicit `Debug` (`[STR-5]`) needs every field to have one, and a `Cell`
# has no format yet.

struct Holder:
    c: Cell[int]

fn main():
    h = Holder(c=Cell(1))
    println(h)
