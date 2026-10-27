#$ test: run-pass
#$ rules: R12, R8
#$ stdout: a
#$ stdout: click
#$ stdout: b
#$ stdout: click
## Verdicts 17 and 22, R12 (rules.md lines 649-650): no function in an `Array[fn()]` may change `LOG`
## (E2113 keeps one out), so calling them inside a loop over `LOG` is allowed (G-07 settled by the type;
## AN-76 part 6, "Accepted (since v2)").

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers:
        for ref mut h in this.handlers:
            h()

fn main() changes LOG:
    LOG.push(String("a"))
    LOG.push(String("b"))
    b = Button()
    b.on_click(fn(): println("click"))
    for entry in LOG:
        println(entry)
        b.click()
