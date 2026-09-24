#$ test: compile-fail
#$ rules: STD-9
#$ error[E0900]: printing a `Point` is not implemented yet
# D-197 — `println` of one value is checked for a printer like several are;
# a value with none used to reach the C compiler as a string. A struct's
# derived `Display` (`[STR-5]`) is not built yet.

struct Point:
    x: int

fn main():
    p = Point(x=1)
    println(p)
