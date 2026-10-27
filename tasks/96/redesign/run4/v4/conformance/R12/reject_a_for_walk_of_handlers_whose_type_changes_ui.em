#$ test: compile-fail
#$ rules: R12, R8
#$ help: handlers: Array[fn(ref mut Ui)]
## R8 (c), R12 (rules.md lines 669-672): handlers that may change any Ui are not run from a `for` walk of
## their list (E3020); line 1013's message, naming `main`'s line too: `main` names `Ui` (R5, lines 349-351),
## which after the fix would be refused, since `fire` then changes only its own Ui (R18, verdict 73).

class Ui:
    handlers: Array[fn(Ui) changes Ui] = []
    current: int = 0
    clicks: int = 0

    fn fire() changes handlers, Ui:
        for ref mut h in this.handlers:
            h(this)    #$ error[E3020]: this loop walks `this.handlers` (line 14), and `h(this)` may change any `Ui`, this one included (the handlers' type says `changes Ui`); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`, with `fn fire() changes this:`; `main` then changes only its own `ui`: write `fn main():`

fn main() changes Ui:
    ui = Ui()
    ui.fire()
    println(ui.clicks)
