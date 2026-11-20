# Clause EFF-25.1: "call `tick()` after the loop: move line 13 below the loop"
#$ test: run-pass
#$ rules: EFF-25, DIA-27, DIA-25
#$ stdout: start
#$ stdout: 2
# The message's fix compiles: the spec's program with `tick()` moved below the loop.

static mut LOG: Array[String] = []

fn note(owns msg: String):
    LOG.push(owns msg)

fn tick():
    note(owns String("tick"))

fn main():
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
    tick()
    println(LOG.len())
