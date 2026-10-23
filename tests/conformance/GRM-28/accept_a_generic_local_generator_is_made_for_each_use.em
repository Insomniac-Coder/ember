#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-12, TYP-16, TYP-17
#$ profiles: debug, development, release
#$ stdout: 1 2 3
#$ a b
#$ 7-1 1-7
#$ x-1 1-x
## `[GRM-28]`, `[GRM-21]` (ODR-268) — a local generator may be generic: each set of type arguments
## its calls infer is a generator of its own (`[TYP-16]`), its body checked once with them opaque
## (`[TYP-17]`). `scaled` calls `each`, which it captures as a closure would. One inside a generic
## function is made for each of that function's instances, capturing `u` there.

fn labelled[U: Display](u: U) -> String:
    gen fn both[T: Display](t: T) -> Generator[String]:
        yield f"{u}-{t}"
        yield f"{t}-{u}"
    got = [s for s in both(1)]
    return f"{got[0]} {got[1]}"

fn main():
    scale = 1
    gen fn each[T: Copy](xs: Array[T]) -> Generator[T]:
        for x in xs:
            yield x
    gen fn scaled(xs: Array[int]) -> Generator[int]:
        for x in each(xs):
            yield x * scale
    got = [v for v in scaled([1, 2, 3])]
    println(got[0], got[1], got[2])
    chars = [c for c in each(['a', 'b'])]
    println(chars[0], chars[1])
    println(labelled(7))
    println(labelled("x"))
