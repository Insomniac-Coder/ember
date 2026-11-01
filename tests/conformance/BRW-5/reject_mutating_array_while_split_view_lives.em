#$ test: compile-fail
#$ rules: BRW-1, BRW-2, BRW-5, SPN-1
#$ error[E3022]: `values` is already mutably borrowed

# Both halves retain the one mutable borrow of the source Array. A structural
# split proves that the halves do not overlap each other; it does not release
# the owner for mutation while either half remains live.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    parts = values.as_mut_span().split_at(1)
    left = parts.0
    values.push(owns 3)
    println(left[0])
