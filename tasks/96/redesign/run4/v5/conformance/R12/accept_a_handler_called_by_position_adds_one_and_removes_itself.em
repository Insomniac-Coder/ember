#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 1 1
## R12 (rules.md lines 768-770), R8 (c) (lines 491-492: "A `while` loop that tests its position against
## the list's current length at every step may add and remove"): AN-60's `Ui`. `fire` walks by position
## and gives each handler the whole Ui `ref mut`; the call copies the handler's two words out first, so
## the list is not an argument and a handler may add handlers (they run in the same dispatch, as in
## Python) or remove itself. `add_once` adds `count`, removes itself, and `count` then runs once. The
## nearest allowed program to reject_a_for_walk_of_handlers_given_the_ui_ref_mut and to
## reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    clicks: int = 0

    fn fire():
        this.current = 0
        while this.current < len(this.handlers):
            this.handlers[this.current](ref mut this)
            this.current += 1

    fn remove_current():
        this.handlers.remove(this.current)
        this.current -= 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn add_once(ref mut ui: Ui):
    ui.handlers.push(count)
    ui.remove_current()

fn main():
    ui = Ui()
    ui.handlers.push(add_once)
    ui.fire()
    println(ui.clicks, len(ui.handlers))
