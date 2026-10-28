#$ test: run-pass
#$ rules: R12, R8
#$ stdout: once
#$ stdout: 8
#$ stdout: 8
## R12 (rules.md lines 639-640; section 2, line 785, "one count on a handler's own state"): "A body that
## may end its own state (it removes itself) and uses that state afterwards holds one count on it across
## that call." The handler removes itself from the list (which ends the list's function value, and so
## its state) and then prints its own `label`. AN-60 part 5's case (`c/an61_dispatch.c` mode 2 clean,
## mode 3 without the count a heap-use-after-free). Checked by stdout, and by AddressSanitizer on the
## emitted C: no report.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    total: int = 0

    fn fire() changes this:
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)
            this.current += 1

    fn remove_current() changes handlers, current:
        this.handlers.remove(this.current)
        this.current -= 1

fn count(ref mut ui: Ui):
    ui.total += 1

fn main():
    ui = Ui()
    label = String("once")
    once_handler = fn(ref mut u: Ui) with owns label:
        for _ in 0..8:
            u.handlers.push(count)
        u.remove_current()
        println(label)
    ui.handlers.push(once_handler)
    ui.fire()
    println(ui.total)
    println(len(ui.handlers))
