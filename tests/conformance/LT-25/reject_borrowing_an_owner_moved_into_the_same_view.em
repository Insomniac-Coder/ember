#$ test: compile-fail
#$ rules: LT-18, LT-25, LT-26, BRW-1, B5, TST-17

# A view must not become self-referential by borrowing a local and then moving
# that local into another field of the same constructed value. Multi-region
# inference grants no exception for the destination's independent field
# slots; the move is reported as shape B5, `E3024` (a field would borrow
# another field of the same value), not as an ordinary move while borrowed.

@view
struct SelfView:
    first: Span[i32]
    values: Array[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    item = SelfView(values.as_span(), values) #$ error[E3024]: `values` would be moved into this value while another of its fields borrows it
    println(item.first[0])
