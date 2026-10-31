#$ test: run-pass
#$ stdout: ['mut', 'read']
#$ ['read', 'read', 'mut']
# A generic extension may name a tuple of its parameters as its target
# (`extend[A: Param, B: Param] (A, B) implements Params:`): every tuple of that arity whose
# elements meet the bounds takes it, as an instance of a generic type takes its extensions. A tuple
# is no package's own, so such an extension implements an interface (`[IFC-2]`).
[T]
struct Mut:
    pass

interface Param:
    static fn describe() -> String

extend[T] Mut[T] implements Param:
    static fn describe() -> String:
        return "mut"

struct Pos:
    x: int

extend Pos implements Param:
    static fn describe() -> String:
        return "read"

interface Params:
    static fn names() -> Array[String]

extend[A: needs Param, B: needs Param] (A, B) implements Params:
    static fn names() -> Array[String]:
        return [A.describe(), B.describe()]

extend[A: needs Param, B: needs Param, C: needs Param] (A, B, C) implements Params:
    static fn names() -> Array[String]:
        return [A.describe(), B.describe(), C.describe()]

[Q: needs Params]
fn names_of() -> Array[String]:
    return Q.names()

fn main():
    println(names_of[(Mut[Pos], Pos)]())
    println(names_of[(Pos, Pos, Mut[Pos])]())
