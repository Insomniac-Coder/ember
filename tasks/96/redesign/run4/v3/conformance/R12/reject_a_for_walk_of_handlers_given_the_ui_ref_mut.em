#$ test: compile-fail
#$ rules: R12, R8
#$ help: while this.current < len(this.handlers):
## R8 (c), R12 (rules.md lines 553-556): a handler given the whole Ui `ref mut` may add to or remove from
## `this.handlers`, so the list may not be walked by a `for` loop around the call (the agreed limitation);
## it is walked by position. AN-60 part 6's message, in this program's lines.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    clicks: int = 0

    fn fire() changes this:
        for ref mut h in this.handlers:
            h(ref mut this)    #$ error[E3020]: this loop walks `this.handlers` (line 14), and `h(ref mut this)` (line 15) gives each handler the whole Ui, so a handler may add to or remove from `this.handlers`; walk by position, as Python's list does: instead of the `for` line and `h(ref mut this)`, write `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)` and `this.current += 1`

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.fire()
    println(ui.clicks)
