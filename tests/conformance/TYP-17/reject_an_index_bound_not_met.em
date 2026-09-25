#$ test: compile-fail
#$ rules: TYP-17, STD-12
# ODR-042 — a `Map[String, int]` is indexed by what is its key: `Index[str]`
# with an `int` `Output`, not `Index[int]`, nor an `Output` of `str`.

fn get[C: Index[int, Output = int]](c: C) -> int:
    return c[0]

fn word[C: Index[str, Output = str]](c: C) -> str:
    return c["a"]

fn main():
    m: Map[String, int] = {"a": 1}
    println(get(m))    #$ error[E2040]: `std.collections.Map[String, i64, std.collections.DefaultHasher]` does not implement `std.core.Index[i64]`, which `C` requires
    println(word(m))    #$ error[E2040]: `std.collections.Map[String, i64, std.collections.DefaultHasher]`'s `Output` for `std.core.Index[str]` is `i64`, but `C`'s bound needs `str`
