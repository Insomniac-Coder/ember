#$ test: compile-fail
#$ rules: BRW-1, BRW-2, BRW-5, SPN-1
#$ error[E3022]: `values` is already mutably borrowed

# Both halves retain the one mutable borrow of the source Array. A structural
# split proves that the halves do not overlap each other; it does not release
# the owner for mutation while either half remains live.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.split_at_mut(1)
    left = parts.0
    values.push(3)
    println(left[0])
