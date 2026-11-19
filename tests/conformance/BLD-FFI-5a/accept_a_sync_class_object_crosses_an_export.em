#$ test: run-pass
#$ rules: BLD-FFI-5a, FFI-31b
#$ profiles: debug, development, release
#$ stdout: 2 2
#$ stdout: 3
#$ stdout: 4 0
#$ assert-c: contains("_Counter_retain(")
#$ assert-c: contains("_Counter_release(")
#$ assert-c: contains("_Counter_bump(")
#$ assert-c: contains("_Shape_sides(")
# `[BLD-FFI-5a]`, `[FFI-31b]` (ODR-162) — an `@export` function may give C and C++ a `thread_shared`
# value of a class, and take one: it crosses with a count of the object (atomic, as a `thread_shared`
# value's are). The library gives C a handle's count and its end, and each `pub` method, which the
# `--cpp` header makes a class of the same name; here the program calls its exports itself. A
# virtual method is called through the object's method table, so C++ reaches an override.

from std.sync import Atomic, MemoryOrder

pub class Counter:
    hits: Atomic[i32] = Atomic(0)

    pub fn bump() -> i32:
        return this.hits.fetch_add(1, MemoryOrder.AcqRel) + 1

    pub fn total() -> i32:
        return this.hits.load(MemoryOrder.Acquire)

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_counter"))
pub fn make_counter() -> thread_shared Counter:
    return thread_shared Counter()

# SAFETY: nothing else in the process defines this name
@unsafe(export("bump_twice"))
pub fn bump_twice(counter: thread_shared Counter) -> i32:
    counter.bump()
    return counter.bump()

pub open class Shape:
    pub virtual fn sides() -> i32:
        return 0

pub class Square(Shape):
    pub override fn sides() -> i32:
        return 4

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_square"))
pub fn make_square() -> thread_shared Shape:
    return thread_shared Square()

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_shape"))
pub fn make_shape() -> thread_shared Shape:
    return thread_shared Shape()

fn main():
    counter = make_counter()
    println(bump_twice(counter), counter.total())
    shared = thread_shared counter
    shared.bump()
    println(counter.total())
    println(make_square().sides(), make_shape().sides())
