#$ test: compile-fail
#$ rules: R12, R4
#$ help: `b.click()`, then `b.handlers.clear()`
## R12 (rules.md lines 676-679): "a function value is not stored in a place that a real variable it
## changes owns, all the way down" (E3022), since it would change its own holder while it runs. AN-53
## part 6; rules.md section 3's line 1008 message, word for word.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn main():
    b = Button()
    b.on_click(fn() changes b: b.handlers.clear())    #$ error[E3022]: this function changes `b`, and `on_click` would keep it in `b.handlers`, so it would change the button that holds it while the button runs it; clear the handlers after the click instead: `b.click()`, then `b.handlers.clear()`
    b.click()
