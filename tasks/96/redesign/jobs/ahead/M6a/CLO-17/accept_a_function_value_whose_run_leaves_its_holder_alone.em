# Clause CLO-17.2: "clear the handlers after the click instead: `b.click()`, then
# `b.handlers.clear()`"
#$ test: run-pass
#$ rules: CLO-17, DIA-25
#$ stdout: hi
#$ stdout: 0
# A handler that changes nothing that holds it is stored; the message's fix compiles.

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(owns f)

    fn click():
        for h in this.handlers:
            h()

fn main():
    b = Button()
    b.on_click(owns fn(): println("hi"))
    b.click()
    b.handlers.clear()
    println(b.handlers.len())
