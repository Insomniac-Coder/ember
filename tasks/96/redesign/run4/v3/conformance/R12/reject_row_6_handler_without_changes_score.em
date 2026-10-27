#$ test: compile-fail
#$ rules: R12, R5
#$ help: b.on_click(fn() changes score: score += 1)
## Row 6, verdict 22, R12 (rules.md lines 505-507): a lambda that changes a real variable it captures
## names it on its first line; a plain `fn()` changes nothing at all, captured variables included.
## AN-53 part 6; the message is rules.md section 3's E2105 row (line 819), word for word.

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
    b.on_click(fn(): score += 1)    #$ error[E2105]: this function changes `score`, a variable it uses from `game`; say so on its first line: `b.on_click(fn() changes score: score += 1)`
    b.click()
    return score

fn main():
    println(game())
