#$ test: compile-fail
#$ rules: CLS-7, EXC-15
#$ error[E2103]: cannot assign to `self` in a class method
#$ help: to re-point a caller's handle, take it as a `mut` parameter [FN-9]
# `[CLS-7]`, `[EXC-15]` (ODR-072) — in a class method `self` names the object the
# method was called on for the whole call. A `mut self` method holds every field of
# that object; re-pointing `self` would leave the rest of the method writing an
# object nobody holds.

class Counter:
    value: int

    fn reset(mut self):
        self = Counter(0)

fn main():
    c = Counter(5)
    c.reset()
    println(c.value)
