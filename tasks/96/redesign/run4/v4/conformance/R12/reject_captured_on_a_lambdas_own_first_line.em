#$ test: compile-fail
#$ rules: R12
#$ help: fn() changes score:
## R12 (rules.md lines 643-644), verdict 22: "A lambda names its own captured variables and does not
## write `captured` (E2105)". The message is rules.md section 3's line 990, word for word.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes captured: score += 1)    #$ error[E2105]: this function changes `score`; a function names the variables it changes: write `fn() changes score:` (`captured` is for a list or a field of functions, which cannot name `score`)
    b.click()
    return score

fn main():
    println(game())
