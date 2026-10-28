#$ test: run-pass
#$ rules: R12, R2, R8
#$ stdout: 2
## R8 (f) and R12 (rules.md lines 462-465, 758-759): a call through a `ref mut` name of a stored function
## value uses the name only to copy the function's two words out, before the arguments are given, so
## `ref mut this`, which reaches `h`'s place, may be given to it; `h` is not used after the call. AN-61's
## second NEW form, and its first (by position).

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire() changes this:
        i = 0
        while i < len(this.handlers):
            h = ref mut this.handlers.get_mut(i).unwrap()
            h(ref mut this)
            i += 1

    fn fire_by_position() changes this:
        i = 0
        while i < len(this.handlers):
            this.handlers[i](ref mut this)
            i += 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.fire()
    ui.fire_by_position()
    println(ui.clicks)
