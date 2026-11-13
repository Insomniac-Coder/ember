#$ test: compile-fail
#$ rules: STR-2
## `[STR-2]` - a field is named only through `this`, which a default has not, since it runs before the
## value exists: a default that reads another field of its type, or `this`, is `E1010` with the `init` to
## write instead.

struct Buf:
    n: int
    cap: int = n * 2    #$ error[E1010]: `cap`'s default reads `n`, a field of the same `Buf`, which has no value when defaults run; set `cap` in an `init`: `fn init(n: int):` with `this.n = n` and `this.cap = n * 2`

struct Timer:
    name: String
    on_tick: fn() -> int = fn() => this.name.len()    #$ error[E1010]: `this` can only be used in a method, and a field default runs before `init`, when there is no object yet; set `on_tick` in `init`

fn main():
    b = Buf(1)
    t = Timer("x")
    println(b.cap, t.on_tick())
