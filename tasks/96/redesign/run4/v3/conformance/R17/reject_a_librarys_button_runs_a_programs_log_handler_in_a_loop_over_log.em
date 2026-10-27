#$ test: compile-fail
#$ rules: R17, R12, R8
#$ help: b.click()
## R17, R12, verdict 40: `ledger`'s `Button` keeps `fn() changes captured`, which cannot name this
## program's `LOG`; the word is filled in from the whole program, every function value stored into that
## type, so `b.click()` changes `LOG` here and is refused inside a loop over `LOG` (R8 (c), E3020).

from ledger import Button

static mut LOG: Array[String] = []

fn main() changes LOG:
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    for line in LOG:
        b.click()    #$ error[E3020]: this loop walks `LOG` (line 16), and `b.click()` (line 17) changes `LOG`: `click` runs the handlers of `b`, an `Array[fn() changes captured]` (its first line says `changes handlers, captured`), which this program fills with a `fn() changes LOG` (line 15); click after the loop
    println(LOG.len())
