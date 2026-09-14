@view
pub struct Pair:
    pub left: Span[i32]
    pub right: Span[i32]

pub fn bundle(left: Span[i32], right: Span[i32]) -> Pair:
    return Pair(left, right)

pub fn right_value(pair: Pair) -> i32:
    return pair.right[0]

pub fn latebound_value(f: @latebound fn(Span[i32]) -> i32, view: Span[i32]) -> i32:
    return f(view)
