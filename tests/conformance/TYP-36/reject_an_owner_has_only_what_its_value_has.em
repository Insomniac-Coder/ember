#$ test: compile-fail
#$ rules: TYP-36
# `[TYP-36]` — an owner has only what the value it owns has, and a `weakref` link has none of it.

struct Point:
    x: int

class Node:
    v: int = 1

[T: needs Ord]
fn lower(a: T, b: T) -> bool:
    return a < b

fn main():
    p = shares Point(1)
    println(lower(shares p, shares p))    #$ error[E2040]: `shares Point` does not implement `Ord`
    n = shares Node()
    w = weakref n
    println(lower(w, w))    #$ error[E2040]: `weakref Node` does not implement `Ord`
