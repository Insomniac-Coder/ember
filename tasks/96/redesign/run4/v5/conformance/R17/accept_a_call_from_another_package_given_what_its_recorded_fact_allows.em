#$ test: run-pass
#$ rules: BLD-14, FN-1
#$ stdout: a
#$ stdout: a
## design rules: R17, R4
## R17, R4, verdict 53: the nearest allowed program, the line E3031's message gives (a copy), and a
## `ledger` function whose recorded fact says it uses `s` before it clears `HISTORY`, given
## `HISTORY[0]` itself.

from ledger import HISTORY, record, record_first

fn main():
    HISTORY.push(String("a"))
    record(copy of HISTORY[0])
    println(HISTORY[0])
    record_first(HISTORY[0])
    println(HISTORY[0])
