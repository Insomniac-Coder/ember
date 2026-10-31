pub const SHIFT: int = 7

pub fn choose(x: int, y: int = x + SHIFT) -> int:
    return x + y

[T: needs Copy]
pub fn mirror(value: T, other: T = value) -> T:
    return other
