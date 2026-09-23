#$ test: compile-fail
#$ rules: CLI-19, GRM-25
#$ error[E0900]: a computed operand in the middle of a comparison chain is not implemented yet
#$ help: bind it to a local first, then compare the local
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning.

fn next() -> int:
    return 5

fn main():
    println(0 < next() < 10)
