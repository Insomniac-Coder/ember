#$ test: run-pass
#$ rules: R5, R4, R12
#$ stdout: 2
## R5, R4: a plain `Atomic` parameter is changed by its own operation, `fetch_add`, and the call takes
## no word (verdict 3); no first line names the change (verdict 86).

from std.sync import Atomic

fn add(total: Atomic[int], n: int):
    total.fetch_add(n)

fn main():
    total = Atomic(0)
    add(total, 2)
    println(total.load())
