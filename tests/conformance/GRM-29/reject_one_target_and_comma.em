#$ test: compile-fail
#$ rules: GRM-29
#$ error[E0100]: a tuple has two or more elements; write `e`
# One target and a comma in a `target_list`.

fn main():
    a = 0
    a, = 1
    println(a)
