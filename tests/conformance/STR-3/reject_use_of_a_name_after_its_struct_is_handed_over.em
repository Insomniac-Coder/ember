#$ test: compile-fail
#$ rules: STR-3
## `[STR-3]` — a struct with a field that is not `Copy` is not `Copy`: `=` from a whole name hands
## it over, and the name is not used again.

struct Named:
    name: String

fn main():
    a = Named(name="ember".to_string())
    b = a
    println(b.name)
    println(a.name)    #$ error[E3040]: `a` is borrowed after it has been moved out of
