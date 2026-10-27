#$ test: compile-pass
#$ rules: TST-6, TST-7, GRM-2
#$
#$ `[TST-6]` — Annex A's code block is generated from this file by `tools/spec_check.py
#$ --emit-appendix`, and this file is compiled (`ember_driver/tests/milestones.rs`,
#$ `annex_a_is_generated_from_a_fixture_that_compiles`), so the quick reference is code that
#$ compiles, not only code that parses. The lines between `#$ begin fixture` and `#$ end fixture`
#$ declare what the printed fragment names and are not printed; every `#$` line is the harness's.
#$ `import c "zlib.h"` reads the header the test writes beside this file.
#$ begin fixture
#$ end fixture
import std.io                                   # module import: binds `io`
import std.thread
from std.math import Vec3, sin as sine          # item import
from std.sync import Atomic, Lock
import c "zlib.h" with (link=["z"]) as zlib     # C header import (Part XVI)

const MAX: int = 1024                           # compile-time constant
static HITS: Atomic[int] = Atomic(0)            # never replaced; changed by its own operations
static NAMES: Array[StrView] = ["ann", "bob"]   # initialised once
static mut FRAME: int = 0                       # a module variable that may change

@derive(Copy)
struct Point:                                   # value type; Eq, Debug, Clone implicit
    x: float
    y: float = 0.0                              # field default
    fn length() -> float:                       # a method: its object is `this`
        return (this.x * this.x + this.y * this.y).sqrt()
    fn scale(k: float):                         # changes `this`, worked out from its body
        this.x *= k
        this.y *= k
    static fn origin() -> Point:                # no object: called `Point.origin()`
        return Point(x=0.0)

struct Mesh:
    points: Array[float]
    fn into_points() owns this -> Array[float]: # takes its object over
        return this.points

open class Script:                              # an object in its own block; subclassable
    let entity: int                             # set only in init
    pub(read) health: float = 100.0             # readable everywhere, written here
    fn init(entity: int):
        this.entity = entity
    virtual fn on_update(dt: float):
        pass

class Door(Script):                             # inherits Script's init
    angle: float = 0.0
    override fn on_update(dt: float):
        this.angle = min(this.angle + 90.0 * dt, 90.0)

class Node:                                     # one owner unless a line says `shares`
    name: String
    children: Array[Node] = []                  # owns its children
    parent: weakref Node = None                 # a link that does not keep its target alive
    mesh: Option[owns Mesh] = None              # held alone, in a block of its own
    fn add(owns child: Node):                   # `owns`: handed over
        child.parent = weakref this
        this.children.push(child)

struct Watcher:                                 # holds a view: lives no longer than its target
    target: sharedref mut Node                  # may change it; other holders may too

class Log:                                      # may be made `thread_shared`
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []   # read and changed only under `lock`
    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

enum Shape:                                     # tagged union
    Circle(r: float)
    Rect(w: float, h: float)
    Empty

interface Drawable:
    fn draw() -> String

extend Shape implements Drawable:
    fn draw() -> String:
        return f"{this!r}"

fn area(s: Shape) -> float:                     # a plain parameter reads the caller's value
    return match s:                             # match expression: `=>` arms
        Circle(r) => 3.14159 * r * r
        Rect(w, h) => w * h
        Empty => 0.0

fn fill(ref mut buf: MutSpan[float], v: float): # `ref mut`: writes the caller's elements
    for ref mut x in buf:                       # the loop's own handle changes the item
        x = v

fn bump(ref mut c: int):                        # changes the caller's variable
    c += 1

fn total(owns xs: Array[int]) -> int:           # `owns`: handed over
    return xs.iter().sum()

fn before(s: StrView, sep: StrView) -> StrView from s:    # the result points into `s` only
    head, _, _ = s.partition(sep)
    return head

fn parse_pair(s: StrView) -> Result[(int, int)]:    # error type defaults to AnyError
    a, _, b = s.partition(",")
    return Ok((a.trim().parse[int]()?, b.trim().parse[int]()?))    # `?` converts ParseError

gen fn countdown(n: int) -> Generator[int]:     # generator
    for i in (0..=n).rev():
        yield i

fn evens(xs: Span[int]) -> some Iterator[Item = int]:    # opaque return type
    return xs.iter().copy_each().filter(fn(x) => x % 2 == 0)

fn inspect(sc: Script):                         # a value of a base class
    if sc is Door as d:                         # kind test: names it `d` when it is a `Door`
        println(d.angle)
    println((cast sc as Door).angle)            # sure: stops the program if it is not one

fn demo(xs: Array[int], m: Map[String, int], opt: Option[Point]):
    squares = [x * x for x in xs if x > 0]      # list comprehension
    total = sum(x * x for x in xs)              # generator expression, no allocation
    index: Map[StrView, int] = {name: i for i, name in NAMES.iter().enumerate()}    # map comprehension
    seen = {1, 2, 3}                            # set literal
    for name, count in m.items():               # a Map iterates keys; items() gives pairs
        println(f"{name}: {count}")
    q = 7 // 2                                  # floor division; `7 / 2` is an error
    if 0 <= q < 10 and q in seen:               # chained comparison, membership
        println(f"{q=} {q:>8}")                 # f-string: `=` form and format spec
    if opt is None:
        return
    if opt exists as p:                         # opens an `Option` or a `weakref` link
        println(p.length())
    if m.get("ann") exists as copy of v:        # a copy of your own, which may change
        v += 1
    label = "big" if q > 3 else "small"         # conditional expression
    r = ref xs[0]                               # a read-only view of a number (`=` copies one)
    n = q
    bump(ref mut n)                             # the call says the argument may change
    ys = copy of xs                             # a copy of your own
    zs = ys                                     # hands `ys` over: `ys` is not used again
    counts: Map[StrView, int] = {"a": 1, "b": 2}
    for k, ref mut c in counts:                 # changes the value under each key
        c *= 2
    double = fn(x: int) => x * 2                # lambda
    add_q = fn(x: int) copy of q => x + q       # keeps its own copy of `q`
    tick = fn(): FRAME += 1                     # no word: changes the real `FRAME`
    tick()
    with scope = thread.scope():                # scoped threads; joins at block end
        scope.spawn(fn() => println(xs.len()))
    h = thread.spawn(fn() copy of q: println(q))    # an unscoped thread names what it keeps
    h.join()
    log = thread_shared Log()                   # one of several owners, on any thread
    t = thread.spawn(fn() thread_shared log: log.push("hi"))
    t.join()
    root = Node(name=String("root"))
    root.add(Node(name=String("leaf"), mesh=Some(owns Mesh(points=[0.0, 1.0]))))
    for ref mut child in root.children:         # may change each child
        child.name = String("child")
    w = weakref root                            # does not keep `root` alive
    if w exists as node:                        # the object still lives: call it `node`
        println(node.name)
    a = Watcher(target=sharedref mut root)
    b = Watcher(target=sharedref mut root)      # two holders may change one node
    a.target.name = String("seen")
    other = Node(name=String("other"))
    second = shares mut other                   # a second owner that may change it
    second.name = String("shared")
    defer: println("leaving demo")              # runs at scope exit
    comptime: assert(mem.size_of[Point]() == 16)    # compile-time check
    unsafe: ptr_write_example()                 # SAFETY: an unsafe block names why it is sound
    @parallel(chunk=256)
    for i in 0..xs.len():
        consume_item(xs[i])
