#$ test: run-pass
#$ rules: R12, R3
#$ stdout: ann
#$ stdout: bo
## R12 (rules.md lines 635-636): a named capture (`copy of`) is the function value's own state, so the
## handler outlives nothing; it changes only its own state, so it is a plain `fn()`.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:
        for ref mut h in this.handlers:
            h()

fn main():
    names = [String("ann"), String("bo")]
    b = Button()
    for name in names:
        b.on_click(fn() with copy of name: println(name))
    b.click()
