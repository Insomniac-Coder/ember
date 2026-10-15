#$ test: run-fail
#$ rules: THR-14
#$ panics: `store` does not support `MemoryOrder.Acquire`
#$ stdout: 2
# `[THR-14]` — an order the compiler cannot see is checked when the operation runs.

from std.sync import Atomic, MemoryOrder

fn chosen(acquire: bool) -> MemoryOrder:
    if acquire:
        return MemoryOrder.Acquire
    return MemoryOrder.Release

fn main():
    a = Atomic(1)
    a.store(2, chosen(false))
    println(a.load(chosen(true)))
    a.store(3, chosen(true))
