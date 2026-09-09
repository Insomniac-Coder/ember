## `[MOD-2]` — "All items are private to their module unless `pub`", fields
## included.
pub struct Box2:
    pub width: i32
    height: i32

pub fn make(w: i32, h: i32) -> Box2:
    return Box2(w, h)

pub fn height_of(b: Box2) -> i32:
    return b.height
