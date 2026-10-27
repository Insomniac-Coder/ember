#$ test: compile-fail
#$ rules: R17, R4
#$ help: record(copy of HISTORY[0])
## R17, R4, verdict 53: across a package's edge the fact "uses `s` after a change" has no word, so
## `ledger`'s interface file records it for `record`, and the call is checked by it: `HISTORY[0]` is
## part of `HISTORY`, which `record` clears before it reads `s` (E3031).

from ledger import HISTORY, record

fn main():
    HISTORY.push(String("a"))
    record(HISTORY[0])    #$ error[E3031]: `record` clears `HISTORY` and then uses `s` (`ledger`'s interface file records it), and `HISTORY[0]` is part of `HISTORY`; give it a copy: `record(copy of HISTORY[0])`
    println(HISTORY[0])
