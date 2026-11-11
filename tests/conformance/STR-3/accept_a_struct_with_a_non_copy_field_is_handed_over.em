#$ test: run-pass
#$ rules: STR-3
#$ stdout: ember
## `[STR-3]` — a struct with a field that is not `Copy` is not `Copy`: `=` from a whole name hands
## it over.

struct Named:
    name: String

fn main():
    a = Named(name="ember".to_string())
    b = a
    println(b.name)
