# Clause EFF-25.1: "A message about a change a call makes names the chain of calls that makes it,
# and no message asks for a word on a first line"
#$ test: compile-fail
#$ rules: EFF-25, DIA-27, BRW-18, EFF-24
# The spec's own program and message, with this file's line numbers.

static mut LOG: Array[String] = []

fn note(owns msg: String):
    LOG.push(owns msg)

fn tick():
    note(owns String("tick"))

fn main():
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        tick()    #$ error[E3020]: `tick()` (line 19) adds to `LOG` (through `note`, line 10) while this loop walks `LOG` (line 17); call `tick()` after the loop: move line 19 below the loop
