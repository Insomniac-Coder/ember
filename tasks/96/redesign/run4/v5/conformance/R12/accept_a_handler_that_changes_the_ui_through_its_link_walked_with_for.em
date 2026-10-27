#$ test: run-pass
#$ rules: R12, R8, R11, R13
#$ stdout: 1
## Run 3's G-05 handler, verdict 86, R12 (rules.md lines 662-670, 717-720), R8 (c): the handler changes
## a `Ui` through its `weakref ui`, which the compiler works out from its body; it changes that Ui's
## `clicks`, not its `handlers`, so the `for` walk of `this.handlers` around `h(this)` changes nothing it
## walks and is allowed. Its neighbour that clears the list is refused
## (reject_a_for_walk_of_handlers_one_of_which_clears_them_through_a_weakref).

class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire():
        for ref mut h in this.handlers:
            h(this)

fn watch(ref mut ui: Ui):
    bump = fn(seen: Ui) weakref ui:
        if ui exists as ref mut me:
            me.clicks += 1
    ui.handlers.push(bump)

fn main():
    ui = Ui()
    watch(ref mut ui)
    ui.fire()
    println(ui.clicks)
