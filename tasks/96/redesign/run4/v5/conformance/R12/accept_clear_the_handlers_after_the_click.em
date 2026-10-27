#$ test: run-pass
#$ rules: R12
#$ stdout: 1 0
## R12 (rules.md lines 722-725): the way out of the nearest refused program
## (reject_a_handler_stored_in_the_button_it_changes): the handlers are cleared after the click, by the
## function that owns the button, not by a handler the button holds.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    b.handlers.clear()
    b.click()
    println(score, b.handlers.len())
