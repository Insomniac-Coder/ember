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
    println(a.name)    #$ error[E3040]: `a` was handed over to `b` on line 11 and is used on line 13; to keep both, write `b = copy of a` (a copy of your own) or `b = shares a` (two owners of one value)
