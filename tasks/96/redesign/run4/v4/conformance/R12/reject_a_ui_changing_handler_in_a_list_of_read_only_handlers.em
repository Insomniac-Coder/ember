#$ test: compile-fail
#$ rules: R12, R11, R13
#$ help: handlers: Array[fn(ref mut Ui)] = []
## R12 (rules.md lines 603-605, 645-649): G-05's handler changes a `Ui` through `with weakref ui`, so its
## type is `fn(Ui) changes Ui`, which a list of `fn(Ui)` (functions that change nothing outside
## themselves) does not take (E2113). AN-60 part 6's message, with the way out that compiles here (R18,
## verdict 73): its `Array[fn(Ui) changes Ui]` would leave `fire`'s `for` walk refused (E3020).

class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire() changes handlers:
        for ref mut h in this.handlers:
            h(this)

fn watch(ref mut ui: Ui):
    bump = fn(seen: Ui) with weakref ui changes Ui:
        if ui exists as ref mut me:
            me.clicks += 1
    ui.handlers.push(bump)    #$ error[E2113]: this handler changes a `Ui` through `with weakref ui` (`fn(Ui) changes Ui`), and `handlers` holds `fn(Ui)`, functions that change nothing outside themselves; give each handler the Ui to change as its parameter: write `handlers: Array[fn(ref mut Ui)] = []`; in `fire`, `fn fire() changes this:` and, instead of the `for` line and `h(this)`, `i = 0`, `while i < len(this.handlers):`, `this.handlers[i](ref mut this)`, `i += 1`; and in `watch`, instead of the three `bump` lines, `bump = fn(ref mut seen: Ui): seen.clicks += 1`

fn main():
    ui = Ui()
    watch(ref mut ui)
    ui.fire()
    println(ui.clicks)
