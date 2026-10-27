#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1 1
## Verdict 86, R12 (rules.md lines 662-672): one `Array[fn()]` keeps row 6's handler, which changes
## `main`'s real `score`, and a handler that pushes to `LOG`; `click` and `main` write nothing. AN-53
## part 6's button.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.on_click(fn(): LOG.push(String("clicked")))
    b.click()
    println(score, LOG.len())
