#$ test: compile-fail
#$ rules: R12
#$ help: handlers: Array[fn() changes captured...]
## Verdict 22 (the owner's refinement), R12 (rules.md lines 525-527, 537-540): `captured` means one
## captured variable, whichever; a function that changes two does not fit (E2113). AN-53 part 6's message,
## word for word (it names every line it replaces, R18).

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn game():
    score = 0
    lives = 3
    b = Button()
    hit = fn() changes score, lives:
        score += 1
        lives -= 1
    b.on_click(hit)    #$ error[E2113]: `fn() changes score, lives` changes 2 captured variables, and `handlers` allows 1 (`Array[fn() changes captured]`); for functions that change several at once write `handlers: Array[fn() changes captured...]`, `fn on_click(owns f: fn() changes captured...) changes handlers:` and `fn click() changes handlers, captured...:`
    b.click()
    println(score, lives)

fn main():
    game()
