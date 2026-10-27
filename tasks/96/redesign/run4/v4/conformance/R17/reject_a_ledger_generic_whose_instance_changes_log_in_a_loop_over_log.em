#$ test: compile-fail
#$ rules: R17, R12, R8
#$ help: each([1], fn(x) changes LOG: LOG.push(String("x")))
## R17, R12: what `ledger`'s generic `each` changes is worked out for each instance, compiled by the
## package that makes it: here its change list `C` is `LOG`, so the call inside a loop over `LOG` is
## refused (R8 (c), E3020; check-V2's fix of matrix row R17-6).

from ledger import each

static mut LOG: Array[String] = []

fn main() changes LOG:
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        each([1], fn(x) changes LOG: LOG.push(String("x")))    #$ error[E3020]: this loop walks `LOG` (line 14), and `each(...)` (line 16) changes `LOG` (the function it is given says `changes LOG`); move `each([1], fn(x) changes LOG: LOG.push(String("x")))` below the loop
    println(LOG.len())
