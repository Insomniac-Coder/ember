#$ test: compile-fail
#$ rules: R12, R2, R8
#$ help: h = ref mut this.handlers.get_mut(i).unwrap()
## R12 (rules.md lines 776-778), R2: a plain name for a part is a read-only view, and calling a stored
## handler that changes its own state (its own copy of `n`, worked out from the only function the program
## stores there) is a change of the place that holds it (E3023). AN-61 part 6's message, with the cause
## named (rules.md section 3's line 1042 shape). The nearest allowed program is
## accept_a_ref_mut_name_from_get_mut_calls_the_handler.

class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    clicks: int = 0

    fn fire():
        i = 0
        while i < len(this.handlers):
            h = this.handlers.get_mut(i).unwrap()
            h(ref mut this)    #$ error[E3023]: `h` only reads the handler in `this.handlers`, and calling it may change its own state (`step`, stored on line 27, adds to its own copy of `n`, line 25); call it by position, `this.handlers[i](ref mut this)` in place of lines 17 and 18, or name it so it may change: `h = ref mut this.handlers.get_mut(i).unwrap()` in place of line 17
            i += 1

fn main():
    ui = Ui()
    n = 0
    step = fn(ref mut u: Ui) copy of n:
        n += 1
        u.clicks += n
    ui.handlers.push(step)
    ui.fire()
    println(ui.clicks)
