#$ test: run-pass
#$ rules: CLO-10, CLO-3, CLO-2, CLO-11
#$ profiles: debug, release, shipping
#$ stdout: 1 2 3
#$ 10
#$ 9
#$ 1 2
#$ 2
#$ end
#$ drop captured

# `[CLO-10]` (D-554) — an owned callable value holds its closure's state: three
# words inline, a larger state in one allocation; a call writes the state in
# place, and the value's end drops what it captured. `[CLO-3]` — a function's
# value converts to one with nothing in it.

class Token:
    name: String

    fn drop(mut self):
        println(f"drop {self.name}")

fn counter() -> fn() -> int:
    n = 0
    return owned fn() -> int:
        n = n + 1
        return n

fn four(a: int, b: int, c: int, d: int) -> fn() -> int:
    return owned fn() => a + b + c + d

fn named() -> fn(int) -> int:
    t = Token("captured")
    return owned fn(x: int) -> int:
        return x + t.name.len()

struct Holder:
    f: fn() -> int

fn main():
    c = counter()
    println(c(), c(), c())
    println(four(1, 2, 3, 4)())
    g = named()
    println(g(1))
    h = Holder(counter())
    println(h.f(), h.f())
    s = "hi".to_string()
    k: fn() -> int = owned fn() => s.len()
    println(k())
    println("end")
