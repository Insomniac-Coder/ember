#$ test: compile-fail
#$ rules: TYP-28
#$ error[E2240]: `/` on two integers
#$ help: write `a // b` for floor division
# `/` is true division; two integer operands, in an expression or a compound
# assignment, are rejected with the floor and float fix-its.

fn main():
    x = 7 / 2
    y = 10
    y /= 2
    println(x + y)
