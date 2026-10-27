#$ test: run-pass
#$ rules: R17, R12, R8
#$ stdout: start
#$ stdout: 2
## R17, R12: the nearest allowed program: the click, which runs the handler this program stored in
## `ledger`'s `Button` and so pushes to `LOG`, comes after the loop over `LOG`; no first line names it
## (verdict 86).

from ledger import Button

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    b = Button()
    b.on_click(fn(): LOG.push(String("clicked")))
    for line in LOG:
        println(line)
    b.click()
    println(LOG.len())
