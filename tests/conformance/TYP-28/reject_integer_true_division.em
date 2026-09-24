#$ test: compile-fail
#$ rules: TYP-28
#$ help: write `a // b` for floor division
# `/` is true division; two integer operands, in an expression or a compound
# assignment, are rejected with the floor and float fix-its.

fn main():
    x = 7 / 2    #$ error[E2240]: `/` on two integers
    y = 10
    y /= 2    #$ error[E2240]: `/` on two integers
    println(x + y)
