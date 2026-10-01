#$ test: run-pass
#$ rules: TYP-16
#$ stdout: 5 3 9 7 8 true
#$ assert-c: contains("em_Wrap_ptr_i64")
#$ assert-c: contains("em_Pair_tup2_i64_i64_i64")
#$ assert-c: contains("em_Pair_i64_tup2_i64_i64")
# `[TYP-16]` (D-412) — each distinct instantiation is a distinct type. An
# instance was found by a name that dropped its arguments' structure, so
# `Wrap[*int]` and `Wrap[int]` were one type (the second had the first's
# field types), and so were `Pair[(int, int), int]` and
# `Pair[int, (int, int)]`, and `Option[*int]` and `Option[int]`. A type
# declared with an instance's spelling (`Wrap_i64`) is a type of its own.

struct Wrap[T]:
    x: T

struct Pair[A, B]:
    a: A
    b: B

struct Wrap_i64:
    y: int

fn unused(w: Wrap[*int]) -> int:
    return 0

fn first(w: Wrap[int]) -> int:
    return w.x

fn left(p: Pair[(int, int), int]) -> int:
    return p.b

fn right(p: Pair[int, (int, int)]) -> int:
    return p.b.0 + p.b.1

fn pointer(o: Option[*int]) -> bool:
    return o is None

fn value(o: Option[int]) -> int:
    return o.unwrap_or(0)

fn declared(w: Wrap_i64) -> int:
    return w.y

fn main():
    println(first(Wrap(x = 5)), left(Pair(a = (1, 2), b = 3)), right(Pair(a = 1, b = (4, 5))), value(Some(7)), declared(Wrap_i64(y = 8)), pointer(None))
