#$ test: compile-fail
#$ rules: ARN-8a, OWN-3, TST-23

struct Bag:
    pub values: Array[i32]

fn main():
    slot: MaybeUninit[Bag] = MaybeUninit[Bag].uninit()
    slot.write(Bag(Array[i32]()))
    unsafe:  # SAFETY(uninit): every slot is written before it is read    #$ warning[L3010]: larger than necessary
        value: Bag = slot.assume_init()
        slot.write(Bag(Array[i32]())) #$ error[E3040]: `slot` is borrowed after it has been moved out of
        println(value.values.len())
