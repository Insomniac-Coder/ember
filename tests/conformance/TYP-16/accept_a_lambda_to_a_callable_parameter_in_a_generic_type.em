#$ test: run-pass
#$ rules: TYP-16, TYP-23, CLO-3
#$ stdout: 1
#$ 1
#$ 2
# D-280 — inside a generic type's own methods, a lambda passed to a method
# that takes a callable (`Array.retain`, the type's own `count_where`)
# takes its parameter type from the call. The method's own type parameters
# were numbered from 0 like the type's, so the two shared a slot: the
# lambda's type was inferred into `T`'s and the call was E2020.

struct Bag[T]:
    items: Array[Option[T]] = []

    fn compact(mut self):
        self.items.retain(fn(e) => e.is_some())

    fn count_where(self, keep: fn(ref Option[T]) -> bool) -> int:
        n = 0
        for e in self.items:
            if keep(e):
                n += 1
        return n

    fn somes(self) -> int:
        return self.count_where(fn(e) => e.is_some())

    fn first_as[U](self, f: fn(ref T) -> U) -> Option[U]:
        for e in self.items:
            match e:
                Some(v):
                    return Some(f(v))
                None:
                    pass
        return None

fn main():
    b = Bag[int]()
    b.items.push(Some(1))
    b.items.push(None)
    println(b.somes())
    b.compact()
    println(b.items.len())
    println(b.first_as(fn(v) => v + 1).unwrap())
