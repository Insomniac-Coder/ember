#$ test: compile-fail
#$ rules: CLS-7, EXC-15, OWN-6
#$ error[E2103]: cannot re-point `self` in a class method
#$ error[E2103]: cannot re-point `self` in a class method
# `[CLS-7]` (ODR-072) — `mem.swap` and `mem.replace` write the place they are
# given, so neither may be given `self` itself (F5).

class Counter:
    value: int

    fn swap_with(mut self, other: Counter):
        o = other
        mem.swap(self, o)
        n = mem.replace(self, Counter(3))

fn main():
    c = Counter(1)
    c.swap_with(Counter(2))
