#$ test: run-pass
#$ rules: TYP-17, TYP-37, STR-5, MOD-5
#$ stdout:
#$ 9 b 2.5
#$ ('s', 's') (4, 4) Less Equal Greater
#$ true false Less Greater b
# A bound is met by what the compiler provides as by a written
# implementation: `Ord` for the numbers (floats by totalOrder, `[TYP-37]`)
# and text, and `Clone` and `Eq` for what it can clone or compare, a
# struct's implicit ones included (`[STR-5]`). D-246. `cmp` gives the
# prelude's `Ordering` (`[MOD-5]`), and a `String` it compares is borrowed.

struct P:
    x: int

fn bigger[T: Ord](owned a: T, owned b: T) -> T:
    return b if a < b else a

fn twice[T: Clone](x: T) -> (T, T):
    return (x.clone(), x.clone())

fn same[T: Eq](a: T, b: T) -> bool:
    return a == b

fn order[T: Ord](a: T, b: T) -> Ordering:
    return a.cmp(b)

one = 1
f = 3.0
s = String.from("b")
println(bigger(3, 9), bigger(String.from("a"), String.from("b")), bigger(2.5, -1.0))
println(twice(String.from("s")), twice(4), one.cmp(2), "a".cmp("a"), f.cmp(-1.0))
println(same(P(1), P(1)), same("x", "y"), order(1, 2), order(s, String.from("a")), s)
