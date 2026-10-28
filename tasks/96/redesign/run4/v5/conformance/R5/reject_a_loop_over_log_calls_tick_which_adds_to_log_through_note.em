#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: call `tick()` after the loop: move line 21 below the loop
## design rules: R5, R8
## R5, R8 (c), verdict 86: no first line says what a function changes; the compiler works it out from the
## bodies, through the calls they make: `tick` calls `note`, which pushes to `LOG`, so `tick()` inside a
## loop over `LOG` is refused (E3020), and the message names the chain, as verdict 86's own example
## does. The nearest program that is allowed is `accept_tick_called_after_the_loop_over_log`.

static mut LOG: Array[String] = []

fn note(msg: String):
    LOG.push(msg)

fn tick():
    note(String("tick"))

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        tick()                 #$ error[E3020]: `tick()` (line 21) adds to `LOG` (through `note`, line 12) while this loop walks `LOG` (line 19)
    println(LOG.len())
