#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: deriving `Component` is not implemented yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning.

@derive(Component)
enum Failure:
    Lost

fn main():
    println(1)
