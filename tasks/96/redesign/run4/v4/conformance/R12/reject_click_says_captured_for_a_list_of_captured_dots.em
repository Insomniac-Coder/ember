#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn click() changes handlers, captured...:
## Verdict 35, R12 (rules.md lines 680-684): a function that runs function values whose type says
## `captured...` says `captured...` on its first line. Message in the shape of rules.md section 3's
## line 989 (`click` without `captured`).

class Button:
    handlers: Array[fn() changes captured...] = []

    fn on_click(owns f: fn() changes captured...) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:    #$ error[E2105]: `click` runs the handlers in `this.handlers`, whose type says `changes captured...`; add it to its first line: `fn click() changes handlers, captured...:`
        for ref mut h in this.handlers:
            h()

fn game():
    score = 0
    lives = 3
    b = Button()
    hit = fn() changes score, lives:
        score += 1
        lives -= 1
    b.on_click(hit)
    b.click()
    println(score, lives)

fn main():
    game()
