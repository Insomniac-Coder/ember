#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: deriving `Error` is not implemented yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning.

@derive(Error)
enum Failure:
    Lost

fn main():
    println(1)
