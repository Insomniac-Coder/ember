#$ test: run-pass
#$ rules: R17, R12, R8
#$ stdout: start
#$ stdout: 2
## R17, R12: the nearest allowed program: the instance of `ledger`'s `each` that changes `LOG` runs
## after the loop over `LOG`.

from ledger import each

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    each([1], fn(x): LOG.push(String("x")))
    println(LOG.len())
