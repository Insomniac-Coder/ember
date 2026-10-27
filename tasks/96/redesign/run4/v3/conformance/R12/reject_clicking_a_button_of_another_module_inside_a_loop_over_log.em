#$ test: compile-fail
#$ rules: R12, R8
#$ help: call `b.click()` after the `for` loop
## Verdict 40, R12 (rules.md lines 530-534): a module's `Button` holding this module's `fn() changes LOG`
## handler makes `b.click()` a change of `LOG` here, so it is refused inside a loop over `LOG` (E3020).
## Message in the shape of rules.md section 3's line 836.

import widgets

static mut LOG: Array[String] = []

fn main() changes LOG:
    LOG.push(String("start"))
    b = widgets.Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    for entry in LOG:
        println(entry)
        b.click()    #$ error[E3020]: this loop walks `LOG` (line 16), and `b.click()` (line 18) changes `LOG`: `click` runs the handlers of `b`, an `Array[fn() changes captured]`, and the handler stored on line 15 changes `LOG` (`fn() changes LOG`); click after the loop
