#$ test: compile-fail
#$ rules: SPN-1, SPN-3, BRW-1, BRW-2
#$ error[E3021]: `values` is borrowed here and mutably borrowed elsewhere

# The child Span keeps the original Array's shared loan alive even though the
# parent Span's final direct use is the split call.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    whole = values.as_span()
    parts = whole.split_at(1)
    left = parts.0
    values.push(3)
    println(left[0])
