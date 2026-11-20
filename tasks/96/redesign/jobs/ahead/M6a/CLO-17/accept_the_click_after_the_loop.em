# Clause CLO-17.1: "storing a function value in a field, a list, a local or an iterator adapter
# changes nothing it changes"; `[BRW-18]`'s fix: "click after the loop: move line 17, `b.click()`,
# below the loop"
#$ test: run-pass
#$ rules: CLO-17, BRW-18, DIA-25
#$ stdout: start
#$ stdout: 2
# The handler is stored before the walk, which changes nothing; the message's fix compiles.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn click():
        for h in this.handlers:
            h()

fn log_click():
    LOG.push(owns String("click"))

fn main():
    b = Button()
    b.handlers.push(owns log_click)
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
    b.click()
    println(LOG.len())
