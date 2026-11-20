# Clause CLS-7.2: `this` "for the whole call"; `[BRW-21]`: "an argument for it that points into a
# place the callee reads or changes by another name while the parameter is used is refused too";
# "`this` included"
#$ test: compile-fail
#$ rules: CLS-7, BRW-21, EXC-15, BCK-8
# `close` clears `UIS`, which ends the `Ui` its parameter names when that `Ui` is in `UIS`, and then
# writes `u.count`. `shut` gives it `this`, so `shut`'s summary has `this` used after a line that
# may end it: the call `UIS[0].shut()` is `E3031`, and `mine.shut()` is allowed. Where the refusal
# stands (at the outer call, not at `close(ref mut this)`) is Q6. No text in the spec: code only.

class Ui:
    count: int = 0

    fn shut():
        close(ref mut this)

static mut UIS: Array[Ui] = []

fn close(ref mut u: Ui):
    UIS.clear()
    u.count += 1

fn main():
    mine = Ui()
    mine.shut()
    println(mine.count)
    UIS.push(owns Ui())
    UIS[0].shut()    #$ error[E3031]
