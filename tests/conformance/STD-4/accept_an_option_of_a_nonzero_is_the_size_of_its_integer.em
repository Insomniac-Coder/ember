#$ test: run-pass
#$ rules: STD-4, TYP-13
#$ stdout: 8 16 1 4 16
#$ stdout: some none
#$ stdout: Some(NonZero(value=5)) None
#$ stdout: true false true
#$ stdout: Some(NonZero(value=-3)) Some(NonZero(value=7)) None
#$ assert-c: contains("typedef em_std_core_NonZero_i64 em_Option_std_core_NonZero_i64;")
# `[TYP-13]`, `[STD-4]` — `Option[NonZero[T]]` is the size of `T`: no
# `NonZero` holds 0, so `None` is kept as 0 and the `Option` needs no tag of
# its own. In every other way it is an `Option`: matched, compared, printed,
# and popped from an array.

from std.core import NonZero

fn describe(o: Option[NonZero[i64]]) -> str:
    match o:
        Some(_):
            return "some"
        None:
            return "none"

fn main():
    println(mem.size_of[Option[NonZero[i64]]](), mem.size_of[Option[i64]](), mem.size_of[Option[NonZero[u8]]](), mem.size_of[Option[NonZero[u32]]](), mem.size_of[Option[NonZero[u128]]]())
    five = NonZero.new(5)
    nothing: Option[NonZero[i64]] = NonZero.new(0)
    println(describe(five), describe(nothing))
    println(five, nothing)
    println(five == NonZero.new(5), five == nothing, nothing == NonZero.new(0))
    xs: Array[NonZero[i32]] = Array()
    xs.push(NonZero[i32].new(7).unwrap())
    xs.push(NonZero[i32].new(-3).unwrap())
    println(xs.pop(), xs.pop(), xs.pop())
