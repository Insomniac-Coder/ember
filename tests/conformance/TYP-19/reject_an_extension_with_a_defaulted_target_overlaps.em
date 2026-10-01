#$ test: compile-fail
#$ rules: TYP-19
# D-414 — `extend[K] Store[K]` is `Store[K, Tag]`'s, so it overlaps
# `extend Store[int]` at `Store[int, Tag]`. The pair was skipped on its
# argument count.

interface Named:
    fn name(self) -> int

struct Tag:
    n: int

struct Store[K, H = Tag]:
    k: K

extend[K] Store[K] implements Named:
    fn name(self) -> int:
        return 1

extend Store[int] implements Named:   #$ error[E2041]: `Store[i64, Tag]` and `Store[K, Tag]` can be one type, and both implement `Named`
    fn name(self) -> int:
        return 2

fn main():
    println(Store(k = 5).name())
