#$ test: run-pass
#$ rules: R17, R12, R8
#$ stdout: start
#$ stdout: 2
## R17, R12: the nearest allowed program: the click comes after the loop, and `main` names `LOG`, which
## the handler it stored in `ledger`'s `Button` changes.

from ledger import Button

static mut LOG: Array[String] = []

fn main() changes LOG:
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn() changes LOG: LOG.push(String("clicked")))
    for line in LOG:
        println(line)
    b.click()
    println(LOG.len())
