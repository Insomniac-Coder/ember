#$ test: compile-fail
#$ rules: R12
#$ help: handlers: Array[fn() changes captured, LOG] = []
## Verdicts 17 and 40, R12 (rules.md lines 634-635, 645-649): where `LOG` is visible a list of functions
## names it by its name; `captured` covers only what the line cannot see, so a `fn() changes LOG` does
## not fit `fn() changes captured` here (E2113). rules.md section 3's line 1040 second message, in this
## program's names (it names `click`'s line too, R18).

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
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))    #$ error[E2113]: this function changes `LOG` (`fn() changes LOG`), and `on_click` takes `fn() changes captured`, functions that change one variable each captured; this file declares `LOG`, so the list names it: write `handlers: Array[fn() changes captured, LOG] = []`, `fn on_click(owns f: fn() changes captured, LOG) changes handlers:` and `fn click() changes handlers, captured, LOG:`
    b.click()
    println(LOG.len())
