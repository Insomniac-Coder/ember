#$ test: compile-fail
#$ rules: SPN-3, BRW-1, BRW-2, BRW-5
#$ error[E3022]: `view` is already mutably borrowed

# Reborrowing does not move the parent MutSpan, but it does reserve its
# exclusive access until the last use of either child view.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    view = values.as_mut_span()
    parts = view.split_at(1)
    left = parts.0
    _again = view.split_at(1)
    println(left[0])
