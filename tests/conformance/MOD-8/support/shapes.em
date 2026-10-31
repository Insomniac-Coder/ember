## `[MOD-8]` — a module glob-imported by the cases beside it.
pub struct Square:
    pub side: int

pub fn area(s: Square) -> int:
    return s.side * s.side

pub fn describe() -> StrView:
    return "shapes"

fn hidden() -> int:
    return 0
