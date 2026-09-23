#$ test: compile-fail
#$ rules: GRM-38
#$ profiles: debug
#$ error[E0100]: a generator expression that is not the only argument needs parentheses

fn main():
    xs = [1, 2]
    println(sum(x for x in xs, 10))
