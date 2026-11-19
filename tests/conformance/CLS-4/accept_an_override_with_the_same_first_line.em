#$ test: run-pass
#$ rules: CLS-4
#$ stdout: 3 7
# `[CLS-4]`: an `override` with the same parameter names, words, types, words on `this` and result is accepted; its body
# may differ, and a further `override` may differ from the one before only as the base's slot allows.

open class Handler:
    virtual fn add(a: i32, b: i32) -> i32:
        return a + b

    virtual fn fill(ref mut out: Array[i32]):
        out[0] += 1

open class Doubler(Handler):
    override fn add(a: i32, b: i32) -> i32:
        return a + b + 1

    override fn fill(ref mut out: Array[i32]):
        out[0] += 2

fn main():
    h: Handler = Doubler()
    println(h.add(1, 1), Doubler().add(3, 3))
