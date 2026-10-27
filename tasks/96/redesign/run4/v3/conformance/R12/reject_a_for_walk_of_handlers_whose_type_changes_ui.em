#$ test: compile-fail
#$ rules: R12, R8
#$ help: handlers: Array[fn(ref mut Ui)]
## R8 (c), R12 (rules.md lines 553-556): handlers whose type says `changes Ui` may change any Ui, this one
## and its `handlers` included, so they are not run from a `for` walk of that list (E3020). rules.md
## section 3's line 836 message, in this program's lines.

class Ui:
    handlers: Array[fn(Ui) changes Ui] = []
    current: int = 0
    clicks: int = 0

    fn fire() changes handlers, Ui:
        for ref mut h in this.handlers:
            h(this)    #$ error[E3020]: this loop walks `this.handlers` (line 14), and `h(this)` may change any `Ui`, this one included (the handlers' type says `changes Ui`); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`, with `fn fire() changes this:`

fn main():
    ui = Ui()
    ui.fire()
    println(ui.clicks)
