#$ test: run-pass
#$ rules: DIA-27, EFF-25, BRW-18
#$ stdout: start
#$ 2
# `[DIA-27]` — the fix the chain message gives, made: `tick()` moved below the loop changes `LOG` once
# the walk has ended.

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
