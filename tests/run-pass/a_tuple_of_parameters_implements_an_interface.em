#$ test: run-pass
#$ stdout: ['mut', 'read']
#$ ['read', 'read', 'mut']
# A generic extension may name a tuple of its parameters as its target
# (`extend[A: Param, B: Param] (A, B) implements Params:`): every tuple of that arity whose
# elements meet the bounds takes it, as an instance of a generic type takes its extensions. A tuple
# is no package's own, so such an extension implements an interface (`[IFC-2]`).
struct Mut[T]:
    pass

interface Param:
    fn describe() -> String

extend[T] Mut[T] implements Param:
    fn describe() -> String:
        return "mut"

struct Pos:
    x: int

extend Pos implements Param:
    fn describe() -> String:
        return "read"

interface Params:
    fn names() -> Array[String]

extend[A: Param, B: Param] (A, B) implements Params:
    fn names() -> Array[String]:
        return [A.describe(), B.describe()]

extend[A: Param, B: Param, C: Param] (A, B, C) implements Params:
    fn names() -> Array[String]:
        return [A.describe(), B.describe(), C.describe()]

fn names_of[Q: Params]() -> Array[String]:
    return Q.names()

fn main():
    println(names_of[(Mut[Pos], Pos)]())
    println(names_of[(Pos, Pos, Mut[Pos])]())
