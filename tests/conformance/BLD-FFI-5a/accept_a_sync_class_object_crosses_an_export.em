#$ test: run-pass
#$ rules: BLD-FFI-5a, FFI-31b
#$ profiles: debug, development, release
#$ stdout: 2 2
#$ stdout: 3
#$ assert-c: contains("_Counter_retain(")
#$ assert-c: contains("_Counter_release(")
#$ assert-c: contains("_Counter_bump(")
# `[BLD-FFI-5a]`, `[FFI-31b]` (ODR-162) — an `@export` function may give C and C++ an object of a
# `@sync` class, and take one: its handle crosses with a count of the object (atomic, as a `@sync`
# class's are). The library gives C a handle's count and its end, and each `pub` method, which the
# `--cpp` header makes a class of the same name; here the program calls its exports itself.

from std.sync import Atomic, MemoryOrder

@sync
pub class Counter:
    hits: Atomic[i32] = Atomic(0)

    pub fn bump(self) -> i32:
        return self.hits.fetch_add(1, MemoryOrder.AcqRel) + 1

    pub fn total(self) -> i32:
        return self.hits.load(MemoryOrder.Acquire)

@export("make_counter")
pub fn make_counter() -> Counter:
    return Counter()

@export("bump_twice")
pub fn bump_twice(counter: Counter) -> i32:
    counter.bump()
    return counter.bump()

fn main():
    counter = make_counter()
    println(bump_twice(counter), counter.total())
    shared = counter
    shared.bump()
    println(counter.total())
