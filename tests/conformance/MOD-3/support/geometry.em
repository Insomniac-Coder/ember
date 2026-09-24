## `[MOD-3]` — a module whose types are named through it.
pub struct Point:
    pub x: int
    pub y: int

pub struct Pair[T]:
    pub first: T
    pub second: T

pub enum Shape:
    Dot
    Line(int)

struct Hidden:
    n: int
