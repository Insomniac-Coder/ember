#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: start
#$ stdout: start
#$ stdout: clicked
## design rules: R12, R5, R8
## Verdicts 48 and 86, R12 (rules.md lines 672-674, 735-739): AN-76's button. The handler type is named
## once with `type` (`type Handler = fn()`); `setup` only stores the handler, which is not a change;
## `report` walks `LOG` and clicks after the walk. No first line names what anything changes. The
## nearest allowed program to reject_clicking_a_log_button_inside_a_loop_over_log.

static mut LOG: Array[String] = []

type Handler = fn()

class Button:
    handlers: Array[Handler] = []

    fn on_click(owns f: Handler):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn setup(ref mut b: Button):
    b.on_click(fn(): LOG.push(String("clicked")))

fn report(ref mut b: Button):
    for entry in LOG:
        println(entry)
    b.click()

fn main():
    LOG.push(String("start"))
    b = Button()
    setup(ref mut b)
    report(ref mut b)
    report(ref mut b)
