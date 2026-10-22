#$ test: compile-fail
#$ rules: LT-1a, LT-1
# `[LT-1a]` (ODR-211) — `@borrows(self.inner)` may name a field whose type is a type parameter,
# with a result that is a view only in some instances (`Option[T]`); it is checked in each
# instance. With `T = int` nothing is borrowed and the body is accepted; with `T = str` the
# result is a view, and returning the receiver's other field `keep`, whose view `@borrows` does
# not name, is `E3062` in that instance.

struct Pick[I, T]:
    inner: I
    keep: T

extend[T: Copy, I: Iterator[Item = T]] Pick[I, T]:
    @borrows(self.inner)
    fn first(mut self) -> Option[T]:
        match self.inner.next():
            Some(x):
                return Some(x)
            None:
                return Some(self.keep)    #$ error[E3062]: `@borrows` names only what `self.inner` borrows

fn main():
    numbers = [1, 2]
    p = Pick(numbers.iter().copied(), 7)
    println(p.first())
    names = ["a", "b"]
    q = Pick(names.iter().copied(), "z")
    println(q.first())
