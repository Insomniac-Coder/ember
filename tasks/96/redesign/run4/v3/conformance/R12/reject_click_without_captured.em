#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn click() changes handlers, captured:
## Verdict 35, R12 (rules.md lines 564-568): `click` runs functions whose type says `changes captured`, so
## its first line says `captured`, the word their type uses. AN-53 part 6; rules.md section 3's line 819
## message, word for word.

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:    #$ error[E2105]: `click` runs the handlers in `this.handlers`, whose type says `changes captured`; add it to its first line: `fn click() changes handlers, captured:`
        for ref mut h in this.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    b.click()
    return score

fn main():
    println(game())
