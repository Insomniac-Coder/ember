#$ test: run-pass
#$ rules: R5, R8
#$ stdout: start
#$ stdout: 2
## R5, R8 (c), verdict 86: the nearest program that is allowed: `tick()`, which adds to `LOG` through
## `note`, runs after the loop over `LOG`; no first line names what either changes.

static mut LOG: Array[String] = []

fn note(msg: String):
    LOG.push(msg)

fn tick():
    note(String("tick"))

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    tick()
    println(LOG.len())
