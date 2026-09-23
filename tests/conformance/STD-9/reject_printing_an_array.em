#$ test: compile-fail
#$ rules: STD-9
#$ error[E0900]: printing a `Array[i64]` is not implemented yet
# D-197 — `println` of one value is checked for a printer like several are;
# an `Array` used to reach the C compiler as a string.

fn main():
    xs: Array[int] = [1, 2]
    println(xs)
