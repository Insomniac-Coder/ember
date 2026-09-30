#$ test: compile-fail
#$ rules: STD-19
#$ profiles: debug
#$ error[E2040]: `std.collections.SpanIter[String]`'s `Item` for `std.core.Iterator` is `ref String`, but `J`'s bound needs `ref i64`
# `[STD-19]` — `chain(other)` needs `other`'s items to be of this iterator's
# item type: numbers then strings is not one iterator.

fn main():
    xs: Array[int] = [1, 2]
    words: Array[String] = [String.from("a")]
    println(xs.iter().chain(words.iter()).count())
