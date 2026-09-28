---

# Annex A — Syntax Quick Reference

Every construct below parses under Part III. The block is generated from the conformance fixture
`docs/spec-source/appendix-a.em` (`[TST-6]`).

```ember,fragment
import std.io                                   # module import: binds `io`
import std.thread
from std.math import Vec3, sin as sine          # item import
import c "zlib.h" with (link=["z"]) as zlib     # C header import (Part XVI)

const MAX: int = 1024                           # compile-time constant
static HITS: Atomic[int] = Atomic(0)            # one global value; Sync
static NAMES: Array[str] = ["ann", "bob"]       # initialised once

@derive(Copy)
struct Point:                                   # value type; Eq, Debug, Clone implicit
    x: float
    y: float = 0.0                              # field default
    fn length(self) -> float:                   # borrowed receiver
        return (self.x * self.x + self.y * self.y).sqrt()
    fn scale(mut self, k: float):               # mutable receiver
        self.x *= k
        self.y *= k

open class Script:                              # reference type, subclassable
    let entity: int                             # assigned only in init
    pub(read) health: float = 100.0             # readable everywhere, written here
    fn init(self, entity: int):
        self.entity = entity
    virtual fn on_update(self, dt: float):
        pass

class Door(Script):                             # inherits Script's init
    angle: float = 0.0
    override fn on_update(self, dt: float):
        self.angle = min(self.angle + 90.0 * dt, 90.0)

@sync
class Counter:                                  # shareable across threads; fields fixed after init
    hits: Atomic[int] = Atomic(0)

enum Shape:                                     # tagged union
    Circle(r: float)
    Rect(w: float, h: float)
    Empty

interface Drawable:
    fn draw(self) -> String

extend Shape implements Drawable:
    fn draw(self) -> String:
        return f"{self!r}"

fn area(s: Shape) -> float:                     # parameters are borrowed by default
    return match s:                             # match expression: `=>` arms
        Circle(r) => 3.14159 * r * r
        Rect(w, h) => w * h
        Empty => 0.0

fn fill(mut buf: MutSpan[float], v: float):     # `mut` parameter: in-out
    for x in buf.iter_mut():
        x = v                                   # writes through `ref mut`

fn total(owned xs: Array[int]) -> int:          # `owned`: moved in
    return xs.iter().sum()

fn parse_pair(s: str) -> Result[(int, int)]:    # error type defaults to AnyError
    a, _, b = s.partition(",")
    return Ok((a.trim().parse[int]()?, b.trim().parse[int]()?))    # `?` converts ParseError

gen fn countdown(n: int) -> Generator[int]:     # generator
    for i in (0..=n).rev():
        yield i

fn evens(xs: Span[int]) -> some Iterator[Item = int]:    # opaque return type
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)

fn demo(xs: Array[int], m: Map[String, int], opt: Option[Point]):
    squares = [x * x for x in xs if x > 0]      # list comprehension
    total = sum(x * x for x in xs)              # generator expression, no allocation
    index = {name: i for i, name in NAMES.iter().enumerate()}    # map comprehension
    seen = {1, 2, 3}                            # set literal
    for name, count in m.items():               # a Map iterates keys; items() gives pairs
        println(f"{name}: {count}")
    q = 7 // 2                                  # floor division; `7 / 2` is an error
    if 0 <= q < 10 and q in seen:               # chained comparison, membership
        println(f"{q=} {q:>8}")                 # f-string: `=` form and format spec
    if opt is None:
        return
    if Some(p) = opt:                           # pattern condition
        println(p.length())
    label = "big" if q > 3 else "small"         # conditional expression
    r = ref xs[0]                               # explicit borrow
    double = fn(x: int) => x * 2                # closure (borrows)
    task = owned fn() => println("done")        # closure (owns its captures)
    with scope = thread.scope():                # scoped threads; joins at block end
        scope.spawn(fn() => println(xs.len()))
    defer: println("leaving demo")              # runs at scope exit
    comptime: assert(mem.size_of[Point]() == 16)    # compile-time check
    unsafe: ptr_write_example()                 # unsafe block
    @parallel(chunk=256)
    for i in 0..xs.len():
        consume_item(xs[i])
```
