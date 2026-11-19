#$ test: run-pass
#$ rules: OWN-6, ERR-4
#$ stdout: Some(3) None
#$ stdout: Some('x') None
#$ stdout: [1, 2] []
#$ stdout: Some(None) None
# `[OWN-6]` — `mem.take` leaves the type's `Default`; `std/src/core.em` gives `Option[T]` and
# `Array[T]` theirs with `extend[T] ... implements Default`, which an instance such as
# `Option[int]` has before any method of it is looked up.

import std.mem as mem

fn main():
    a: Option[int] = Some(3)
    b = mem.take(ref mut a)
    println(b, a)
    s: Option[String] = Some(String.from("x"))
    t = mem.take(ref mut s)
    println(t, s)
    xs = [1, 2]
    ys = mem.take(ref mut xs)
    println(ys, xs)
    n: Option[Option[int]] = Some(None)
    m = mem.take(ref mut n)
    println(m, n)
