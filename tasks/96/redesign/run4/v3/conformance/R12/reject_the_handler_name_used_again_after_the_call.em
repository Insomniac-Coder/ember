#$ test: compile-fail
#$ rules: R12, R8
#$ help: h = ref mut this.handlers.get_mut(i).unwrap()
## R8 (a), (f) and R12 (rules.md lines 382-385): a later use of the `ref mut` name of a stored function
## value is checked against what the call may have done; the handler may remove handlers, so `h` may
## name another one (E3060). AN-61 part 6's message, in this program's lines.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire() changes this:
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()
            h(ref mut this)
            h(ref mut this)    #$ error[E3060]: `h(ref mut this)` (line 16) lets the handler remove handlers, which may put another handler in the place `h` names (line 15), and `h` is used on line 17; name it again after the call: `h = ref mut this.handlers.get_mut(i).unwrap()`
            i += 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.fire()
    println(ui.clicks)
