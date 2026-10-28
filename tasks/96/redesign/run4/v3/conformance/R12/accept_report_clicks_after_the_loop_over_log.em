#$ test: run-pass
#$ rules: R12, R5, R8
#$ stdout: start
#$ stdout: start
#$ stdout: clicked
## Verdicts 17 and 48, R12 (rules.md lines 553-556, 583-588): AN-76's button. The handler type is named
## once with `type`; `click` names `LOG`; `setup` only stores the handler, which is not a change, so it
## names nothing; `report` walks `LOG` and clicks after the walk, and names `LOG`.

static mut LOG: Array[String] = []

type Handler = fn() changes LOG

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, LOG:
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button):
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))

fn report(ref mut b: Button) changes LOG:
    for entry in LOG:
        println(entry)
    b.click()

fn main() changes LOG:
    LOG.push(String("start"))
    b = Button()
    setup(ref mut b)
    report(ref mut b)
    report(ref mut b)
