#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: deriving `Default` is not implemented yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning.

@derive(Default)
enum Failure:
    Lost

fn main():
    println(1)
