#$ test: run-pass
#$ rules: TYP-16, TYP-18, STD-11
#$ stdout: 1 2 3
#$ stdout: 4 4 Either.Right(r='r')
# `generic_param := IDENT [":" bound_list] ["=" type]` — an omitted trailing
# type argument takes its parameter's default, as `Map[K, V]` means
# `Map[K, V, DefaultHasher]` (`[STD-11]`). The default is resolved where the
# declaration is and may name an earlier parameter; `Holder[i32]` and
# `Holder[i32, DefaultHasher]` are one type.

from std.collections import DefaultHasher

struct Holder[K, H = DefaultHasher]:
    k: K

struct Pair[A, B = A]:
    a: A
    b: B

enum Either[L, R = L]:
    Left(l: L)
    Right(r: R)

fn show(h: Holder[i32]) -> i32:
    return h.k

fn main():
    a: Holder[i32] = Holder[i32](1)
    b: Holder[i32, DefaultHasher] = Holder[i32, DefaultHasher](2)
    println(a.k, show(b), show(Holder(3i32)))
    p = Pair[int](4, 4)
    e: Either[String] = Either[String].Right("r")
    println(p.a, p.b, e)
