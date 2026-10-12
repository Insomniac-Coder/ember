#$ test: compile-fail
#$ rules: GRM-28, PHIL-12
## `[GRM-28]` — what is not built yet is refused, never ignored: a generic
## local function, and a call to a local function from a closure inside it
## (which would have to capture it).

fn main():
    fn ident[T](x: T) -> T:    #$ error[E0900]: a generic local function is not supported yet
        return x
    println(ident(1))
    fn outer(n: int) -> int:
        fn inner(m: int) -> int:
            return outer(m)    #$ error[E0900]: calling `outer` from a closure inside it is not supported yet
        return inner(n)
    println(outer(1))
