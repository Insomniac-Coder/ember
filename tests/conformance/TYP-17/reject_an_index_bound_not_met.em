#$ test: compile-fail
#$ rules: TYP-17, STD-12
# ODR-042 — a `Map[String, int]` is indexed by what is its key: `Index[str]`
# with an `int` `Output`, not `Index[int]`, nor an `Output` of `str`.

[C: needs Index[int, Output = int]]
fn get(c: C) -> int:
    return c[0]

[C: needs Index[StrView, Output = StrView]]
fn word(c: C) -> StrView:
    return c["a"]

fn main():
    m: Map[String, int] = {"a": 1}
    println(get(m))    #$ error[E2040]: `std.collections.Map[String, i64, std.collections.DefaultHasher]` does not implement `Index[i64]`, which `C` requires
    println(word(m))    #$ error[E2040]: `std.collections.Map[String, i64, std.collections.DefaultHasher]`'s `Output` for `Index[StrView]` is `i64`, but `C`'s bound needs `StrView`
