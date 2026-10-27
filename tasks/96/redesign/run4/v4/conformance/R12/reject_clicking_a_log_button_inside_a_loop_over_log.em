#$ test: compile-fail
#$ rules: R12, R8
#$ help: click after the loop
## Verdict 17, R8 (c), R12 (rules.md lines 669-672): a call through a function value whose type says
## `changes LOG` is a change of `LOG`, refused inside a loop over `LOG` (E3020; run 3's F-04 hole).
## rules.md section 3's line 1013 message, in this program's lines.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn() changes LOG] = []

    fn on_click(owns f: fn() changes LOG) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, LOG:
        for ref mut h in this.handlers:
            h()

fn report(ref mut b: Button) changes LOG:
    for entry in LOG:
        println(entry)
        b.click()    #$ error[E3020]: this loop walks `LOG` (line 21), and `b.click()` (line 23) changes `LOG`: `click` runs the handlers of `b`, an `Array[fn() changes LOG]` (its first line says `changes handlers, LOG`); click after the loop

fn main() changes LOG:
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    report(ref mut b)
