#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: click after the loop: move line 19, `b.click()`, below the loop
## design rules: R17, R12, R8
## R17, R12, verdict 86: `ledger`'s `Button` keeps plain `fn()` values; what a call through them changes
## is worked out from every function the whole program stores there, this program's handler included,
## so `b.click()` pushes to `LOG` here and is refused inside a loop over `LOG` (R8 (c), E3020), the
## message naming the chain.

from ledger import Button

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn(): LOG.push(String("clicked")))
    for line in LOG:
        println(line)
        b.click()    #$ error[E3020]: `b.click()` (line 19) adds to `LOG` (through `click`, which runs the handler stored on line 16, which pushes to `LOG`) while this loop walks `LOG` (line 17)
    println(LOG.len())
