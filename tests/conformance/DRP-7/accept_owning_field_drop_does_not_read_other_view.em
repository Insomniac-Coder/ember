#$ test: run-pass
#$ rules: DRP-6, DRP-7
#$ stdout: 1
# Plain has an owning field but no drop of its own. Dropping that field does
# not read the unrelated borrowed field, even through a generic parameter.

struct Plain[T]:
    borrowed: ref i32
    payload: T

fn main():
    source: i32 = 5
    text: String = "owned"
    _holder = Plain(ref source, text)
    source = 6
    other: i32 = 7
    more: String = "also owned"
    _wrapped: Option[Plain[String]] = Some(Plain(ref other, more))
    other = 8
    println(1)
