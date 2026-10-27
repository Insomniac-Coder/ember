#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1 1
## R12 (rules.md lines 527-528, 535, 564-568): `Array[fn() changes captured, LOG]` keeps row 6's handler
## and a `LOG` handler; `click` says the words of its list's type, and `main`, which wrote the lambdas,
## names only `LOG` (its `score` is its own). AN-53 part 6's way out.

static mut LOG: Array[String] = []

class Button:
    handlers: Array[fn() changes captured, LOG] = []

    fn on_click(owns f: fn() changes captured, LOG) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, captured, LOG:
        for ref mut h in this.handlers:
            h()

fn main() changes LOG:
    score = 0
    b = Button()
    b.on_click(fn() changes score: score += 1)
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    b.click()
    println(score, LOG.len())
