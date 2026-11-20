# Clause CLO-2.1: "A name a lambda or a local function uses from the function that made it is the
# **real variable**, that very variable and not a copy, unless a capture word after its brackets
# names it"; "So `b.on_click(owns fn(): score += 1)` changes the real `score`, and `b.on_click(owns
# fn() copy of score: println(score))` keeps a copy of its own."
#$ test: run-pass
#$ rules: CLO-2, CLO-4, CLO-18
#$ stdout: 0
#$ stdout: 1
# The spec's two handlers: the first changes the real `score`; the second prints the copy it made
# when `score` was 0.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(owns f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    score = 0
    b = Button()
    b.on_click(owns fn(): score += 1)
    b.on_click(owns fn() copy of score: println(score))
    b.click()
    println(score)
