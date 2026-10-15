#$ test: compile-fail
#$ rules: THR-14
#$ error[E7006]: `store` does not support `MemoryOrder.Acquire`
#$ error[E7006]: `load` does not support `MemoryOrder.Release`
#$ error[E7006]: `load` does not support `MemoryOrder.AcqRel`
# `[THR-14]` — a load has no release half and a store no acquire half: such an order is `E7006`.

from std.sync import Atomic, MemoryOrder

fn main():
    a = Atomic(1)
    a.store(2, MemoryOrder.Acquire)
    println(a.load(MemoryOrder.Release))
    println(a.load(order=MemoryOrder.AcqRel))
