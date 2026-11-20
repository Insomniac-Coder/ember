# Clause CLO-4.1: "the value must provably end before them (it is kept only in locals, local lists or
# objects that never leave the function, and every function it is handed to keeps it no longer, by
# that function's summary, `[EFF-24]`)"; and the message's fix
#$ test: run-pass
#$ rules: CLO-4, CLO-5, EFF-24, DIA-25
#$ stdout: 3
#$ stdout: 2
# Lambdas using the real `total`, kept in a local, a local list and a callee that runs them and keeps
# nothing; and the message's lines, a counter the handler owns a share of.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(owns f)

    fn click():
        for ref mut h in this.handlers:
            h()

class Counter:
    n: int = 0

fn run(f: fn()):
    f()

fn register(ref mut b: Button, owns count: shares mut Counter):
    b.on_click(owns fn() owns count: count.n += 1)

fn main():
    total = 0
    one = fn(): total += 1
    one()
    fs: Array[fn()] = []
    fs.push(owns fn(): total += 1)
    for ref mut f in fs:
        f()
    run(fn(): total += 1)
    println(total)
    b = Button()
    count = Counter()
    register(ref mut b, shares mut count)
    b.click()
    b.click()
    println(count.n)
