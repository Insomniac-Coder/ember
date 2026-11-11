#$ test: compile-fail
#$ rules: THR-6, OWN-6

# `[THR-6]` — a `@must_drop` value's drop is a safety guarantee: it may not be
# stored in a class field or an `owns` block or collection (a struct's field
# included), captured by an `owned fn`, or returned from the function that made
# it. Each is `E3015`, shape O8. (Nothing forgets a value: `mem.forget` is gone,
# `[OWN-6]`.)

@must_drop
struct Guard:
    id: int

    fn drop():
        println(f"released {this.id}")

class Holder:
    g: Guard    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be stored in the class field `g`

struct Pack:
    many: Array[Guard]    #$ error[E3015]: `Array[Guard]` would hold a `@must_drop` value

fn make() -> Guard:
    g = Guard(1)
    return g    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be returned from the function that made it

fn main():
    b: owns Guard = owns Guard(3)    #$ error[E3015]: `owns Guard` would hold a `@must_drop` value
    h = Guard(4)
    f = owned fn() => h.id    #$ error[E3015]: `Guard` holds a `@must_drop` value, which may not be captured by an `owned fn` (`h`)
