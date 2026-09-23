#$ test: compile-fail
#$ rules: GRM-27
#$ profiles: debug
#$ error[E1010]: cannot find `x` in this scope
# The loop variables are scoped to the comprehension.

fn main():
    squares = [x * x for x in range(3)]
    println(x)
