#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1 2
## Verdicts 22 and 35, R12 (rules.md lines 632-634, 680-684): `captured...` takes a function that
## changes several captured variables at once, and `click`, which runs such functions, says
## `captured...`, the word its list's type uses; `game` wrote the lambda and names nothing.

class Button:
    handlers: Array[fn() changes captured...] = []

    fn on_click(owns f: fn() changes captured...) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured...:
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
