#$ test: compile-fail
#$ rules: TYP-36, TYP-17
## `[TYP-36]` — a tuple is `Ord` only if every element is, and a reference implements an interface
## only as its referent does (ODR-181): a float is not `Hash`, a struct without `@derive(Ord)` is
## not `Ord` (D-820).

import std.collections

struct Point:
    x: int

fn digest[T: Hash](a: T) -> u64:
    h = collections.DefaultHasher.new()
    a.hash(h)
    return h.finish()

fn main():
    pts: Array[(int, Point)] = [(1, Point(2))]
    pts.sort()  #$ error[E2040]: `(i64, Point)` does not implement `Ord`
    f = 1.5
    println(digest(ref f))  #$ error[E2040]: `ref f64`
