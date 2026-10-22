#$ test: compile-fail
#$ rules: GRM-28, PHIL-12
## `[GRM-28]` — what is not built yet is refused, never ignored: a generic
## local function.

fn main():
    fn ident[T](x: T) -> T:    #$ error[E0900]: a generic local function is not supported yet
        return x
    println(ident(1))
