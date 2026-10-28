#$ test: compile-fail
#$ rules: R12, R11, R13
#$ help: handlers: Array[fn(Ui) changes Ui]
## R12 (rules.md lines 507-509, 537-541): G-05's handler changes a `Ui` through `with weakref ui`, so its
## type is `fn(Ui) changes Ui`, which a list of `fn(Ui)` (functions that change nothing outside
## themselves) does not take (E2113). AN-60 part 6's message, word for word.

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
    ui.handlers.push(bump)    #$ error[E2113]: this handler changes a `Ui` through `with weakref ui` (`fn(Ui) changes Ui`), and `handlers` holds `fn(Ui)`, functions that change nothing outside themselves; give it the Ui to change as its parameter, `fn(ref mut Ui)`, or keep it in a list that says so, `handlers: Array[fn(Ui) changes Ui]`

fn main():
    ui = Ui()
    watch(ref mut ui)
    ui.fire()
    println(ui.clicks)
