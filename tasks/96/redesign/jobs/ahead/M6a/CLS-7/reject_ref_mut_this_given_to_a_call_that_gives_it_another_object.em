# Clause CLS-7.2: "Giving `this` another object in a class method is `E2103`"; "a function that
# gives a caller's place another object takes it as a `ref mut` parameter (`[FN-9]`)"
#$ test: compile-fail
#$ rules: CLS-7, FN-9, EFF-25
# `renew` gives its `ref mut` parameter another object (`[FN-9]`), so `renew(ref mut this)` would
# give `this` another object. This file follows reading (A) of Q1: `E2103`, decided by `renew`'s
# summary; `fix` gives `this` to `bump`, which only writes a field, and is allowed. The spec's
# `E2103` text is for `this = Node()`: code only.

class Ui:
    count: int = 0

    fn reset():
        renew(ref mut this)    #$ error[E2103]

    fn fix():
        bump(ref mut this)

fn renew(ref mut u: Ui):
    u = Ui()

fn bump(ref mut u: Ui):
    u.count += 1

fn main():
    ui = Ui()
    ui.fix()
    ui.reset()
