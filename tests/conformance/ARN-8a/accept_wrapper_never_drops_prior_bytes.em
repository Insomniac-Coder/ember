#$ test: run-pass
#$ rules: ARN-8, ARN-8a, OWN-2, TST-23
# `write` is not assignment and is not `Cell.set`: it stores without dropping
# prior bytes. Only the second Bag becomes ordinary owned `T` through
# `assume_init`, so only that Bag is destroyed.

struct Bag:
    pub values: Array[i32]

    fn drop(mut self):
        println(self.values.len())

fn make(n: i32) -> Bag:
    values: Array[i32] = Array[i32]()
    i: i32 = 0
    while i < n:
        values.push(i)
        i = i + 1
    return Bag(values)

fn main():
    slot: MaybeUninit[Bag] = MaybeUninit[Bag].uninit()
    slot.write(make(1))
    slot.write(make(2))
    unsafe:
        _value: Bag = slot.assume_init()
        println(99)
#$ stdout: 99
#$ 2
