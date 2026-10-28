#$ test: compile-fail
#$ rules: R12, R2, R8
#$ help: h = ref mut this.handlers.get_mut(i).unwrap()
## R12 (rules.md lines 641-644), R2: a plain name for a part is a read-only view, and calling a stored
## handler through it may change the handler's own state (E3023). AN-61 part 6's message, word for word.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire() changes this:
        i = 0
        while i < len(this.handlers):
            h = this.handlers.get_mut(i).unwrap()
            h(ref mut this)    #$ error[E3023]: `h` only reads the handler, and calling it may change its own state; call it by position: `this.handlers[i](ref mut this)`, or name it `h = ref mut this.handlers.get_mut(i).unwrap()`
            i += 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.fire()
    println(ui.clicks)
