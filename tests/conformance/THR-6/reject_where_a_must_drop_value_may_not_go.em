#$ test: compile-fail
#$ rules: THR-6, OWN-6

# `[THR-6]` — a `@must_drop` value's drop is a safety guarantee: it may not be
# forgotten, stored in a class field or a `Box` or collection (a struct's field
# included), captured by an `owned fn`, or returned from the function that made
# it. Each is `E3015`, shape O8.

import std.mem

@must_drop
struct Guard:
    id: int

    fn drop(mut self):
        println(f"released {self.id}")

class Holder:
    g: Guard    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be stored in the class field `g`

struct Pack:
    many: Array[Guard]    #$ error[E3015]: `Array[Guard]` would hold a `@must_drop` value

fn make() -> Guard:
    g = Guard(1)
    return g    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be returned from the function that made it

fn main():
    g = Guard(2)
    mem.forget(g)    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be forgotten
    b = Box(Guard(3))    #$ error[E3015]: `Box[Guard]` would hold a `@must_drop` value
    h = Guard(4)
    f = owned fn() => h.id    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be captured by an `owned fn` (`h`)
