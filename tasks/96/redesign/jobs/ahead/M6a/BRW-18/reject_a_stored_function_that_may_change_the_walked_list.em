# Clause BRW-18.1: "nothing changes the collection, whatever the change (...) through the
# collection's own name or through any call that may change it (`[EFF-1]`: a call through a
# function value (...))"; "For a call through a stored function that may change the walked list:
# E3020"
#$ test: compile-fail
#$ rules: BRW-18, CLO-17, EFF-1, DIA-27
# The `E3020` is the spec's message, with this file's line numbers. `close_all` is put in the list
# directly, so "stored on line N" is that line under either reading of Q2. Storing it is `E3022` by
# `[CLO-17]`'s letter (it clears, by its own name, the list that holds it): Q4, which also asks how
# this message's fix can compile. This file follows the letter. Code only for the `E3022`.
# The other calls that change a walked list are in EFF-26 (functions given to a call, adapters),
# CLO-17 (a library's stored handler) and DRP-8 (an end that runs a `drop`); `spawn` is M9's and a
# foreign call's callbacks M12's.

class Ui:
    handlers: Array[fn(Ui)] = []
    current: int = 0

    fn run_all():
        for h in this.handlers:
            h(this)    #$ error[E3020]: `h(this)` (line 21) may clear `this.handlers` (through `close_all`, stored on line 30) while this loop walks `this.handlers` (line 20); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`

static mut UI: Ui = Ui()

fn close_all(u: Ui):
    println(u.current)
    UI.handlers.clear()

fn main():
    UI.handlers.push(owns close_all)    #$ error[E3022]
    UI.run_all()
