#$ test: run-pass
#$ rules: R12
#$ stdout: 1 0
## R12 (rules.md lines 560-563): the way out of the nearest refused program: the handlers are cleared
## after the click, by the function that owns the button, not by a handler the button holds.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn main():
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    b.click()
    b.handlers.clear()
    b.click()
    println(score, b.handlers.len())
