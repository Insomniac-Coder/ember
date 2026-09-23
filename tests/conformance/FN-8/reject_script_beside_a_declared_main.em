#$ test: compile-fail
#$ rules: FN-8, GRM-2
#$ error[E1030]: `main` is already declared in this module
# A file with statements at file scope that also declares `main` has two items
# named `main`.

fn main():
    println(1)

println(2)
