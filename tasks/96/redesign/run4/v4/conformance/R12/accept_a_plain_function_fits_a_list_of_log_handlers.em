#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1
## Verdict 17, R12 (rules.md lines 645-649): a function that changes less than the list's type allows
## fits it ("a plain `fn()` in an `Array[fn() changes LOG]`"); `main` runs them and names `LOG`.

static mut LOG: Array[String] = []

fn log_click() changes LOG:
    LOG.push(String("clicked"))

fn quiet():
    pass

class Button:
    handlers: Array[fn() changes LOG] = []

fn main() changes LOG:
    b = Button()
    b.handlers.push(log_click)
    b.handlers.push(quiet)
    for ref mut h in b.handlers:
        h()
    println(LOG.len())
