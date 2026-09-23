#$ test: compile-fail
#$ rules: ATT-1
#$ profiles: debug
#$ error[E0104]: `@frobnicate` is not an attribute
# An attribute that is not in the table, and is not an `@attribute` struct, is
# `E0104`: never accepted and ignored.

@frobnicate
fn main():
    println(1)
