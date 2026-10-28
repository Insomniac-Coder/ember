#$ test: compile-fail
#$ rules: R5, R12
#$ help: fn add(total: Atomic[int], n: int) changes Atomic[int]:
## R5, verdict 29: an `Atomic` is changed through any name by its own operations, and the change is
## still named on the first line (`changes Atomic[int]`, E2105).

from std.sync import Atomic

fn add(total: Atomic[int], n: int):
    total.fetch_add(n)    #$ error[E2105]: `add` changes `total` through `fetch_add`; say so on its first line: `fn add(total: Atomic[int], n: int) changes Atomic[int]:`

fn main():
    total = Atomic(0)
    add(total, 2)
    println(total.load())
