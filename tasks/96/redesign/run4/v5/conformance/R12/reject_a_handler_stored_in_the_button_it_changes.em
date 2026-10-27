#$ test: compile-fail
#$ rules: R12, R4
#$ help: clear the handlers after the click instead
## R12 (rules.md lines 722-725): "a function value is not stored in a place that a real variable it
## changes owns, all the way down: `b.on_click(fn(): b.handlers.clear())` ... [is] refused (E3022), since
## the function would change its own holder while it runs, as Rust refuses". AN-53 part 6; rules.md
## section 3's line 1060 last message, in this program's lines. The nearest allowed program is
## accept_clear_the_handlers_after_the_click.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    b = Button()
    b.on_click(fn(): b.handlers.clear())    #$ error[E3022]: this function clears `b.handlers`, and `on_click` would keep it in `b.handlers`, so it would change the button that holds it while the button runs it; clear the handlers after the click instead: remove line 22 and write `b.handlers.clear()` below line 23, `b.click()`
    b.click()
