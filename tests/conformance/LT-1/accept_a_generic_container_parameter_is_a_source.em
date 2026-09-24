#$ test: run-pass
#$ rules: LT-1, TYP-16
#$ stdout: 4
#$ ab
# The source set comes from the declared signature, so a generic container
# parameter is a source for every instantiation.

fn first_of[T](xs: Array[T]) -> ref T:
    return ref xs[0]

fn main():
    xs = [4, 5]
    println(first_of(xs))
    names: Array[String] = ["ab", "c"]
    println(first_of(names))
