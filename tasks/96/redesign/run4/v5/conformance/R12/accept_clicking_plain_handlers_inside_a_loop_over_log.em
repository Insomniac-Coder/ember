#$ test: run-pass
#$ rules: CLO-1, BRW-16
#$ stdout: a
#$ stdout: click
#$ stdout: b
#$ stdout: click
## design rules: R12, R8
## R12 (rules.md lines 665-670): "calling what a list holds whose functions change nothing is allowed
## inside such a loop". The only handler the program stores in `Button.handlers` prints, so `b.click()`
## changes nothing in `LOG` and runs inside the loop over `LOG` (G-07, now settled by what the program
## stores, verdict 86; AN-76 part 6, "Accepted (since v2)").

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn main():
    LOG.push(String("a"))
    LOG.push(String("b"))
    b = Button()
    b.on_click(fn(): println("click"))
    for entry in LOG:
        println(entry)
        b.click()
