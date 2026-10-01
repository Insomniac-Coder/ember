#$ test: run-pass
#$ rules: TYP-36, TYP-17
#$ stdout: () 0 None
# `[TYP-36]` — the table gives `void` the `Default` `()`: a `T: Default`
# bound accepts `void`, and `T.default()` is `()` there (D-463). `void` cannot
# be extended in source, so the compiler answers it, as it does `void`'s
# `Eq`, `Hash` and `Debug`.

fn made[T: Default]() -> T:
    return T.default()

fn main():
    nothing: void = made[void]()
    println(nothing, made[int](), made[Option[void]]())
