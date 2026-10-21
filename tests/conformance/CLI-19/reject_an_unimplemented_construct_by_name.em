#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: a generic local function is not supported yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning.

fn main():
    fn ident[T](x: T) -> T:
        return x
    println(ident(1))
