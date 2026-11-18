# Clause LT-25: "A view field may not point into another field of the value that contains it (shape
# B5)."
#$ test: compile-fail
#$ rules: LT-25, TYP-15, DIA-7
# Shape B5's code, `E3024` (§XVII.6.1); the spec gives no message text. Made so by the constructor,
# and by a later line. A constructor's argument takes no `owns` (`[FN-2a]`).

struct SelfView:
    first: Span[int]
    values: Array[int]

fn made_so():
    values = [7]
    item = SelfView(first=values.as_span(), values=values)    #$ error[E3024]
    println(item.first[0])

fn set_so():
    other = [1]
    item = SelfView(first=other.as_span(), values=[7])
    item.first = item.values.as_span()    #$ error[E3024]
    println(item.first[0], other)

fn main():
    made_so()
    set_so()
