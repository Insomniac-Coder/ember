#$ test: run-pass
#$ rules: CLO-1, BRW-16
#$ stdout: 1
#$ stdout: 2
#$ stdout: 16
#$ stdout: 0
## design rules: R12, R8
## R12 (rules.md lines 768-770; section 2, line 986, "two words copied out of the list"): "A call through
## a list element or a field copies the two words out before the arguments are given and keeps no pointer
## into the list or field", so a handler that pushes 8 handlers (the list's buffer moves) while it runs,
## and changes its own state, writes nothing into the old buffer. AN-61's hole (today's compiler writes
## the state back through a stale pointer, `c/an61_dispatch.c` mode 1, heap-use-after-free). Checked by
## stdout, and by a build of the emitted C under AddressSanitizer: no report.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    total: int = 0

    fn fire():
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)
            this.current += 1

fn count(ref mut ui: Ui):
    ui.total += 1

fn main():
    ui = Ui()
    runs = 0
    grow = fn(ref mut u: Ui) copy of runs:
        runs += 1
        if runs == 1:
            for _ in 0..8:
                u.handlers.push(count)
        println(runs)
    ui.handlers.push(grow)
    ui.fire()
    ui.fire()
    println(ui.total)
    println(runs)
