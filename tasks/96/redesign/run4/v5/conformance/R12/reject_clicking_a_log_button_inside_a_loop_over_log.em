#$ test: compile-fail
#$ rules: R12, R8
#$ help: move line 25, `b.click()`, below the loop
## Verdicts 17 and 86, R8 (c), R12 (rules.md lines 665-670, 717-720): a call through a function value
## makes the changes of the functions the program can put there; the button holds a handler that pushes
## to `LOG`, so `b.click()` is a change of `LOG`, refused inside a loop over `LOG` (E3020; run 3's F-04
## hole). rules.md section 3's line 1065 `b.click()` message, in this program's lines (F-12). The nearest
## allowed program is accept_report_clicks_after_the_loop_over_log.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn()] = []

    fn on_click(owns f: fn()):
        this.handlers.push(f)

    fn click():
        for ref mut h in this.handlers:
            h()

fn report(ref mut b: Button):
    for entry in LOG:
        println(entry)
        b.click()    #$ error[E3020]: `b.click()` (line 25) adds to `LOG` (through `click`, line 18, which runs the handler stored on line 30, which pushes to `LOG`) while this loop walks `LOG` (line 23); click after the loop: move line 25, `b.click()`, below the loop

fn main():
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn(): LOG.push(String("clicked")))
    report(ref mut b)
