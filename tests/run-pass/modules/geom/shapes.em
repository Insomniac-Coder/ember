pub struct Point:
    x: i32
    y: i32

pub fn area(w: i32, h: i32) -> i32:
    return w * h

# Private to this module: `[MOD-2]` makes an item private unless `pub`.
fn hidden() -> i32:
    return 99
