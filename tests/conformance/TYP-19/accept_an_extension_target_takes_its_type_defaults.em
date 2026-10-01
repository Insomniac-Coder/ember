#$ test: run-pass
#$ rules: TYP-19, TYP-17
#$ stdout: 1 2
# D-414 — an extension target that leaves out a defaulted type argument
# means the default, as a type written so does: `extend[K] Store[K]` is
# `Store[K, Tag]`'s. It applied to nothing (`Store[bool]` had no `name`).

struct Tag:
    n: int

struct Store[K, H = Tag]:
    k: K

extend[K] Store[K]:
    fn name(self) -> int:
        return 1

extend Store[int]:
    fn size(self) -> int:
        return 2

fn main():
    s: Store[bool] = Store(k = true)
    t: Store[int] = Store(k = 5)
    println(s.name(), t.size())
