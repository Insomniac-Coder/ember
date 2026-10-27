#$ test: compile-fail
#$ rules: BRW-17
#$ help: name it again after the call
## design rules: R12, R8
## R8 (a), (f) and R12 (rules.md lines 517-520): "a later use of the name is checked against what the call
## may have done (rule (a))"; the only handler the program stores clears the handlers (worked out from
## its body, verdict 86), so after the call `h` may name another handler or none (E3060). AN-61 part 6's
## message, in this program's lines.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire():
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()
            h(ref mut this)
            h(ref mut this)    #$ error[E3060]: `h(ref mut this)` (line 17) may remove handlers (it may run `close_all`, stored on line 26, which clears `ui.handlers`, line 22), which may put another handler in the place `h` names (line 16), or none, and `h` is used on line 18; name it again after the call: `h = ref mut this.handlers.get_mut(i).unwrap()`
            i += 1

fn close_all(ref mut ui: Ui):
    ui.handlers.clear()

fn main():
    ui = Ui()
    ui.handlers.push(close_all)
    ui.fire()
    println(ui.clicks)
