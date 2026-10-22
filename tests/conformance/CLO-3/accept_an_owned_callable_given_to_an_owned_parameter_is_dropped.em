#$ test: run-pass
#$ rules: CLO-3, CLO-6, DRP-1
#$ profiles: debug, development, release
#$ stdout: dropped 1
#$ stdout: called 41
#$ stdout: stored 12
#$ stdout: dropped 2
#$ stdout: dropped 3
#$ stdout: dropped 4
#$ stdout: once 4
#$ stdout: dropped 5
#$ stdout: once 105
#$ stdout: never called
#$ stdout: lent 6
#$ stdout: end
#$ stdout: dropped 6
# D-880 — an owned callable value given to an `owned` parameter is the callee's: it ends with the
# callee, or with what the callee stored it in, and its captured state's `drop` runs then, once.
# Given to a `once fn` parameter (`[CLO-6]`), the call consumes it and ends it (it was never
# dropped), and one the callee never calls ends with the callee. A named one given to a generic
# callable parameter is lent (ODR-240) and stays the caller's, to its scope's end.

struct Noisy:
    value: i32

    fn drop(mut self):
        println("dropped", self.value)

fn make(n: i32) -> fn(i32) -> i32:
    noisy = Noisy(n)
    return owned fn(x: i32) -> i32 => x + noisy.value

fn call_owned(owned f: fn(i32) -> i32, x: i32) -> i32:
    return f(x)

struct Holder:
    f: fn(i32) -> i32

fn keep(owned f: fn(i32) -> i32) -> Holder:
    return Holder(f)

fn finish(owned h: Holder):
    pass

fn call_once(f: once fn(i32) -> i32, x: i32) -> i32:
    return f(x)

fn never_call(f: once fn(i32) -> i32):
    pass

fn call_generic[F: fn(i32) -> i32](owned f: F, x: i32) -> i32:
    return f(x)

fn main():
    f = make(1)
    r = call_owned(f, 40)
    println("called", r)
    h = keep(make(2))
    println("stored", h.f(10))
    finish(h)
    never_call(make(3))
    once = call_once(make(4), 0)
    println("once", once)
    m = Noisy(5)
    passed = call_once(owned fn(x: i32) -> i32 => x + m.value, 100)
    println("once", passed)
    println("never called")
    g = make(6)
    println("lent", call_generic(g, 0))
    println("end")
