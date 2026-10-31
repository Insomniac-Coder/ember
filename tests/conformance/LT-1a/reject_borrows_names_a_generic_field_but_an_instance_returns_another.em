#$ test: compile-fail
#$ rules: LT-1a, LT-1
# `[LT-1a]` (ODR-211) — `@borrows(self.inner)` may name a field whose type is a type parameter,
# with a result that is a view only in some instances (`Option[T]`); it is checked in each
# instance. With `T = int` nothing is borrowed and the body is accepted; with `T = str` the
# result is a view, and returning the receiver's other field `keep`, whose view `@borrows` does
# not name, is `E3062` in that instance.

[I, T]
struct Pick:
    inner: I
    keep: T

extend[T: needs Copy, I: needs Iterator[Item = T]] Pick[I, T]:
    fn first() -> Option[T] from inner:
        match this.inner.next():
            Some(x):
                return Some(x)
            None:
                return Some(this.keep)    #$ error[E3062]: the returned view points into `this`, but its first line says `from inner`

fn main():
    numbers = [1, 2]
    p = Pick(numbers.iter().copy_each(), 7)
    println(p.first())
    names = ["a", "b"]
    q = Pick(names.iter().copy_each(), "z")
    println(q.first())
