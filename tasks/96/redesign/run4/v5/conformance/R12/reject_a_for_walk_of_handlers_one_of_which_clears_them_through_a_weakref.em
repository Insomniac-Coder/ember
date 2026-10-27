#$ test: compile-fail
#$ rules: R12, R8, R11, R13
#$ help: handlers: Array[fn(ref mut Ui)] = []
## Run 3's hole G-05, verdict 86, R8 (c) and R12 (rules.md lines 717-720): a handler typed `fn(Ui)` that
## clears the handlers of the `Ui` its `weakref ui` links to; the compiler works out that change from its
## body (no type says it), so `h(this)` may clear the list this `for` loop walks (E3020). rules.md section
## 3's line 1065 eighth message, in this program's lines; it also names `close_all`'s lines, since a
## `fn(Ui)` does not fit a list of `fn(ref mut Ui)` (R18: the fix compiles). The nearest allowed program
## is accept_a_handler_called_by_position_adds_one_and_removes_itself.

class Ui:
    handlers: Array[fn(Ui)] = []
    current: int = 0

    fn fire():
        for ref mut h in this.handlers:
            h(this)    #$ error[E3020]: `h(this)` (line 17) may clear `this.handlers` (through `close_all`, stored on line 23, which clears the `handlers` of the `Ui` its `weakref ui` links to, line 22) while this loop walks `this.handlers` (line 16); give each handler the Ui to change and walk by position: write `handlers: Array[fn(ref mut Ui)] = []` in place of line 12; instead of lines 16 and 17, `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)` and `this.current += 1`; and instead of lines 20 to 22, `close_all = fn(ref mut seen: Ui): seen.handlers.clear()`

fn watch(ref mut ui: Ui):
    close_all = fn(seen: Ui) weakref ui:
        if ui exists as ref mut me:
            me.handlers.clear()
    ui.handlers.push(close_all)

fn main():
    ui = Ui()
    watch(ref mut ui)
    ui.fire()
    println(len(ui.handlers))
