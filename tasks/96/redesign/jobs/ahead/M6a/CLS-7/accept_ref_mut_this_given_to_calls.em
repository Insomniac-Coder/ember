# Clause CLS-7.2: "Inside a class method `this` is the object the method was called on, for the
# whole call"; `[BRW-18]`'s line, "`this.handlers[this.current](ref mut this)`"
#$ test: run-pass
#$ rules: CLS-7, BRW-21, EXC-15, BRW-18, FN-9
#$ stdout: 2
#$ stdout: 1
#$ stdout: 1
# The case std.ecs needs (W8): a method gives `this` to `ref mut` parameters, directly and through
# stored functions called by position, which change its fields. None of them ends or replaces the
# object, so every call is allowed. `this.handlers[this.current](...)` calls an element (Q8).

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    count: int = 0
    seen: Array[int] = []

    fn run_all():
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)
            this.current += 1

    fn bump_twice():
        bump(ref mut this)
        bump(ref mut this)

fn bump(ref mut u: Ui):
    u.count += 1

fn note(ref mut u: Ui):
    u.seen.push(owns u.current)

fn main():
    ui = Ui()
    ui.bump_twice()
    println(ui.count)
    ui.handlers.push(owns bump)
    ui.handlers.push(owns note)
    ui.run_all()
    println(ui.seen[0])
    println(ui.seen.len())
