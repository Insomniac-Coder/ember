#$ test: run-pass
#$ rules: TYP-15, LT-3
#$ stdout: ann 2 bob
#$ stdout: (1, 'x') 3
## `[TYP-15]` (D-777) — a `static` may hold a view whose every region is `static`: a struct, a
## tuple or an array built only of string literals and values with no region.

struct Named:
    name: str
    age: int

static FIRST: Named = Named("ann", 2)
static PAIR: (int, str) = (1, "x")
static NAMES: Array[Named] = [Named("bob", 3)]

fn main():
    println(FIRST.name, FIRST.age, NAMES[0].name)
    println(PAIR, NAMES[0].age)
