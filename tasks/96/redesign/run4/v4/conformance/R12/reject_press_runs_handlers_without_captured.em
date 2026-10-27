#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn press(ref mut b: Button) changes captured:
## R12 (rules.md lines 680-685): a function that runs functions whose type says `captured` "itself or
## through a call whose first line says it" says so too; only the function that wrote the lambdas needs
## no clause. Message in the shape of rules.md section 3's line 986 (`tick` calls `note`).

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn press(ref mut b: Button):    #$ error[E2105]: `press` calls `b.click()`, which runs handlers whose type says `changes captured` (`click`'s first line says `changes handlers, captured`); add it to its first line: `fn press(ref mut b: Button) changes captured:`
    b.click()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    press(ref mut b)
    return score

fn main():
    println(game())
