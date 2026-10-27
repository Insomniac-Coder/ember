#$ test: compile-fail
#$ rules: R17, R12, R8
#$ help: call it after the loop: move line 16 below the loop
## R17, R12: what `ledger`'s generic `each` changes is worked out for each instance, compiled by the
## package that makes it: here the function it is given pushes to `LOG`, so the call inside a loop over
## `LOG` is refused (R8 (c), E3020; check-V2's fix of matrix row R17-6), the message naming the chain.

from ledger import each

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        each([1], fn(x): LOG.push(String("x")))    #$ error[E3020]: `each(...)` (line 16) adds to `LOG` (through the function it is given, which pushes to `LOG`) while this loop walks `LOG` (line 14)
    println(LOG.len())
