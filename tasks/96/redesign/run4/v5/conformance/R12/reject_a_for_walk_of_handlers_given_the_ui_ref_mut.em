#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: while this.current < len(this.handlers):
## design rules: R12, R8
## R8 (c), R12 (rules.md lines 662-670, 717-720): a handler given the whole Ui `ref mut` that adds to and
## removes from `this.handlers` (`add_once`, worked out from its body, verdict 86), so the list may not be
## walked by a `for` loop around the call (the agreed limitation); it is walked by position. AN-60 part
## 6's message, in this program's lines, naming the chain (R18). The nearest allowed program is
## accept_a_handler_called_by_position_adds_one_and_removes_itself.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    clicks: int = 0

    fn fire():
        for ref mut h in this.handlers:
            h(ref mut this)    #$ error[E3020]: `h(ref mut this)` (line 17) may add to and remove from `this.handlers` (through `add_once`, stored on line 32, which pushes to `ui.handlers` on line 27 and calls `remove_current` on line 28) while this loop walks `this.handlers` (line 16); walk by position, as Python's list does: instead of lines 16 and 17, write `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)` and `this.current += 1`

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
