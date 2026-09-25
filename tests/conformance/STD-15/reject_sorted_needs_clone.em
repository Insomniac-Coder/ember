#$ test: compile-fail
#$ rules: STD-15, STR-5
# `[STD-15]` — `sorted` returns a new array, so its elements must be `Clone`;
# a type with its own `drop` is not, unless it says so (`[STR-5]`, ODR-026).
# `sort` sorts in place and needs only `Ord`.

from std.core import Ordering, Ord, Eq

struct Handle:
    id: int

extend Handle:
    fn drop(mut self):
        pass

extend Handle implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.id == other.id

extend Handle implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        return self.id.cmp(other.id)

fn main():
    hs = [Handle(2), Handle(1)]
    hs.sort()
    copy = hs.sorted()           #$ error[E2040]: `Handle` does not implement `Clone`, which `sorted` needs
