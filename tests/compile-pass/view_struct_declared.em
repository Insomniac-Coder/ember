#$ test: compile-pass
#$ rules: TYP-14

## The same struct with the attribute the rule asks for.

@view
struct Window:
    at: ref i32
    step: i32

fn main():
    println(1)
