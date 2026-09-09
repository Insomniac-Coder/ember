pub struct Point:
    # `[MOD-2]` — a field is private to its module unless it says
    # otherwise, and `[STR-1]` makes the memberwise constructor `pub`
    # only when every field is. `main.em` both constructs a `Point` and
    # reads its fields, so both are `pub`.
    pub x: i32
    pub y: i32

pub fn area(w: i32, h: i32) -> i32:
    return w * h

# Private to this module: `[MOD-2]` makes an item private unless `pub`.
fn hidden() -> i32:
    return 99
