#$ test: compile-fail
#$ rules: R12
#$ help: handlers: Array[fn() changes captured, LOG]
## Verdicts 17 and 40, R12 (rules.md lines 527-528, 537-541): where `LOG` is visible a list of functions
## names it by its name; `captured` covers only what the line cannot see, so a `fn() changes LOG` does
## not fit `fn() changes captured` here (E2113). AN-53 part 6's message, word for word.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn() changes captured] = []

    fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

fn main() changes LOG:
    b = Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))    #$ error[E2113]: this function changes `LOG` (`fn() changes LOG`), and `on_click` takes `fn() changes captured`, functions that change at most the one variable they captured; keep it in a list that says so: `handlers: Array[fn() changes captured, LOG]`, with `fn on_click(owns f: fn() changes captured, LOG) changes handlers:`
    b.click()
    println(LOG.len())
