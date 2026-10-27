#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1 2
## Verdict 86, R12 (rules.md lines 662-672, 752-756): a handler that changes two of `game`'s real
## variables is kept in a button's `Array[fn()]` like any other function; neither the list, `click` nor
## `game` writes anything about what it changes.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn game():
    score = 0
    lives = 3
    b = Button()
    hit = fn():
        score += 1
        lives -= 1
    b.on_click(hit)
    b.click()
    println(score, lives)

fn main():
    game()
