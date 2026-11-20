# Clause CLO-18.1 and CLO-18.2: "walk the handlers with `for ref mut h in this.handlers:`"; "write
# `fn run(ref mut f: fn()):` and call `run(ref mut cb)`"; `[FN-2]`: "`apply(ref mut counter)`"
#$ test: run-pass
#$ rules: CLO-18, FN-2, CLO-10
#$ stdout: done
# Each changing call goes through a name that may change the function value: the lines the two
# messages give compile.

class Button:
    handlers: Array[fn()] = []

    fn click():
        for ref mut h in this.handlers:
            h()

fn run(ref mut f: fn()):
    f()

[F: needs fn()]
fn apply(owns f: F):
    f()

fn main():
    count = 0
    b = Button()
    b.handlers.push(owns fn() copy of count: count += 1)
    b.click()
    b.click()
    score = 0
    cb = fn() copy of score: score += 1
    run(ref mut cb)
    run(ref mut cb)
    n = 0
    counter = fn() copy of n: n += 1
    apply(ref mut counter)
    println("done")
