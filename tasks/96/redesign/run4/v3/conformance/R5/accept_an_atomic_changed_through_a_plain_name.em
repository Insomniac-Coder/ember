#$ test: run-pass
#$ rules: R5, R4, R12
#$ stdout: 2
## R5, R4, verdict 29: a plain `Atomic` parameter is changed by its own operation, `fetch_add`; the call
## takes no word, and the first line names the change. `main` names nothing: `total` is its own local.

from std.sync import Atomic

fn add(total: Atomic[int], n: int) changes Atomic[int]:
    total.fetch_add(n)

fn main():
    total = Atomic(0)
    add(total, 2)
    println(total.load())
