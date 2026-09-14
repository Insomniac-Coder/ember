#$ test: run-pass
#$ rules: LT-18, LT-25, LT-26, BRW-1, TST-17
#$ stdout: 7

# A view may borrow a local owner when the owner remains outside the view. The
# rejected companion only forbids moving that same owner into another field of
# the constructed view while its borrow is live.

@view
struct BorrowedView:
    first: Span[i32]
    marker: i32

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    item = BorrowedView(values.as_span(), 1)
    println(item.first[0])
