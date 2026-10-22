#$ test: run-pass
#$ rules: EXP-4, CTL-3b
#$ profiles: debug, development, release
#$ stdout: 0 3
#$ 1 4
#$ item 1
#$ item 2
#$ drop 1
#$ drop 2
#$ after the shared loop
#$ item 11
#$ item 12
#$ drop 11
#$ drop 12
#$ after the mutable loop
#$ 0 a
#$ 1 b
#$ 2 3 1
## `[EXP-4]` — "one created by a `for` iterable lives to the end of the loop":
## `make().iter()`, with or without adapters, and `make().iter_mut()` view the
## array `make()` returns, which is dropped when the loop ends, not before its
## first element (it was `E2140`, D-816). Within one statement a temporary's
## iterator, `first()` and chunks may be used freely.

struct Noisy:
    n: int

    fn drop(mut self):
        println("drop", self.n)

fn make() -> Array[int]:
    return [3, 4]

fn noisy() -> Array[Noisy]:
    return [Noisy(1), Noisy(2)]

fn names() -> Array[String]:
    return ["a", "b"]

fn main():
    for i, x in make().iter().enumerate():
        println(i, x)
    for x in noisy().iter():
        println("item", x.n)
    println("after the shared loop")
    for x in noisy().iter_mut():
        x.n += 10
        println("item", x.n)
    println("after the mutable loop")
    for i, s in names().iter().enumerate():
        println(i, s)
    println(make().iter().count(), make().first().unwrap() + 0, make().chunks(2).count())
