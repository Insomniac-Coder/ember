#$ test: compile-fail
#$ rules: R12
#$ help: handlers: Array[fn() changes LOG]
## Verdict 17, R12 (rules.md lines 522-523, 537-541): a named function is a value of the type its first
## line gives, and a function value that may change more than the type of the list it is given to is
## refused (E2113). The message is rules.md section 3's line 860, word for word.

static mut LOG: Array[String] = []

fn log_click() changes LOG:
    LOG.push(String("clicked"))

class Button:
    handlers: Array[fn()] = []

fn main() changes LOG:
    b = Button()
    b.handlers.push(log_click)    #$ error[E2113]: `log_click` changes `LOG` (`fn() changes LOG`), and `handlers` holds `fn()`, functions that change nothing outside themselves; keep it in a list that says so: `handlers: Array[fn() changes LOG]`
    for ref mut h in b.handlers:
        h()
