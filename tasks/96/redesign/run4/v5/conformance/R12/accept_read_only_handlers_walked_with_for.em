#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 3
#$ stdout: 3
## R12 (rules.md lines 665-670), R8 (c): the only function the program puts in `handlers` is `show`, which
## changes nothing (worked out from its body, verdict 86), so no handler can change `this.handlers` while
## the `for` loop walks them, and the walk is allowed (AN-60 part 6, the second way out).

class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 3

    fn fire():
        for ref mut h in this.handlers:
            h(this)

fn show(u: Ui):
    println(u.clicks)

fn main():
    ui = Ui()
    ui.handlers.push(show)
    ui.handlers.push(show)
    ui.fire()
