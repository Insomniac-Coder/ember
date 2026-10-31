#$ test: run-pass
#$ rules: THR-8, THR-9, THR-10, THR-11, TYP-22
#$ profiles: debug, development, release
#$ stdout: 9 4
#$ stdout: 12
# `[THR-8]`, `[THR-9]` (ODR-255) — a `dyn I` holds a value of some type implementing `I`: it is
# `Send` when `I` requires `Send` (`interface Shape: Send:`), so every implementation is `Send`,
# and `Sync` when `I` requires `Sync`. A `Box[dyn Shape]` moves to another thread, and two scoped
# tasks read one `Box[dyn Reading]` at once, `Reading` requiring `Send` and `Sync` through a
# parent interface.

import std.thread

interface Shape: needs Send:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Bar:
    width: int
    height: int

extend Bar implements Shape:
    fn area() -> int:
        return this.width * this.height

interface Shared: needs (Send, Sync):
    pass

interface Reading: needs Shared:
    fn value() -> int

struct Gauge:
    level: int

extend Gauge implements Shared:
    pass

extend Gauge implements Reading:
    fn value() -> int:
        return this.level

fn main():
    shapes: Array[Box[dyn Shape]] = [Box(Square(3)), Box(Bar(2, 2))]
    h = thread.spawn(owned fn() -> Array[int] => [s.area() for s in shapes])
    areas = h.join()
    println(areas[0], areas[1])
    r: Box[dyn Reading] = Box(Gauge(6))
    with scope = thread.scope():
        a = scope.spawn(fn() => r.value())
        b = scope.spawn(fn() => r.value())
        println(a.join() + b.join())
