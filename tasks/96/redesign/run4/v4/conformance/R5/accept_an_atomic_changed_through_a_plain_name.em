#$ test: run-pass
#$ rules: R5, R4, R12
#$ stdout: 2
## R5, R4, verdict 29: a plain `Atomic` parameter is changed by its own operation, `fetch_add`; the call
## takes no word, and the first line names the change. `main` names it too: a change a callee's first
## line names by a type counts for its caller, its own local included (R5, verdict 36's example).

from std.sync import Atomic

fn add(total: Atomic[int], n: int) changes Atomic[int]:
    total.fetch_add(n)

fn main() changes Atomic[int]:
    total = Atomic(0)
    add(total, 2)
    println(total.load())
