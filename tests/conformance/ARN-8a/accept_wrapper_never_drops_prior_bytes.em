#$ test: run-pass
#$ rules: ARN-8, ARN-8a, OWN-2, TST-23
# `write` is not assignment and is not `Cell.set`: it stores without dropping
# prior bytes. Only the second Bag becomes ordinary owned `T` through
# `assume_init`, so only that Bag is destroyed.

struct Bag:
    pub values: Array[i32]

    fn drop():
        println(this.values.len())

fn make(n: i32) -> Bag:
    values: Array[i32] = Array[i32]()
    i: i32 = 0
    while i < n:
        values.push(owns i)
        i = i + 1
    return Bag(values)

fn main():
    _value: Bag
    # `[ARN-8]` — a `Bag` needs drop, so `uninit()` and `write` are `unsafe` operations.
    unsafe:  # SAFETY(uninit): the first Bag is written over and never ended, which is what this test shows; the slot is read only after the second write
        slot: MaybeUninit[Bag] = MaybeUninit[Bag].uninit()
        slot.write(owns make(1))
        slot.write(owns make(2))
        _value = slot.assume_init()
    println(99)
#$ stdout: 99
#$ 2
