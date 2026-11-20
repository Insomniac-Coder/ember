# Clause DIA-27.1: "A message about a change a call makes names the chain of calls that makes it
# ("`tick()` (line 10) adds to `LOG` (through `note`, line 4) while this loop walks `LOG` (line 9);
# call `tick()` after the loop: move line 10 below the loop")."
#$ test: compile-fail
#$ rules: DIA-27, EFF-25, BRW-18
# The spec's message for one link, with this file's line numbers. For two links (`step` calls
# `tick`) the spec gives no form; Q9: the claim stops where the forms could differ.

static mut LOG: Array[String] = []

fn note(owns msg: String):
    LOG.push(owns msg)

fn tick():
    note(owns String("tick"))

fn step():
    tick()

fn main():
    for line in LOG:
        tick()    #$ error[E3020]: `tick()` (line 22) adds to `LOG` (through `note`, line 12) while this loop walks `LOG` (line 21); call `tick()` after the loop: move line 22 below the loop
        println(line)
    for line in LOG:
        step()    #$ error[E3020]: `step()` (line 25) adds to `LOG` (through `tick`
        println(line)
