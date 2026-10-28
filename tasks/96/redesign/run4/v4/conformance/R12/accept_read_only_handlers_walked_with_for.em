#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 3
#$ stdout: 3
## Verdict 17, R12 (rules.md lines 645-650): a `fn(Ui)` changes nothing outside itself, so no handler in
## the list can change any Ui's `handlers` while the `for` loop walks them; the walk is accepted for every
## program (AN-60 part 6, the second way out).

class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 3

    fn fire() changes handlers:
        for ref mut h in this.handlers:
            h(this)

fn show(u: Ui):
    println(u.clicks)

fn main():
    ui = Ui()
    ui.handlers.push(show)
    ui.handlers.push(show)
    ui.fire()
