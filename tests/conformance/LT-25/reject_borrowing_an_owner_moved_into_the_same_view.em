#$ test: compile-fail
#$ rules: LT-18, LT-25, LT-26, BRW-1, B5, TST-17

# A view must not become self-referential by borrowing a local and then moving
# that local into another field of the same constructed value. This is rejected
# by the ordinary live-loan move rule; multi-region inference grants no
# exception for the destination's independent field slots.

@view
struct SelfView:
    first: Span[i32]
    values: Array[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    item = SelfView(values.as_span(), values) #$ error[E3021]: `values` cannot be moved while it is borrowed
    println(item.first[0])
