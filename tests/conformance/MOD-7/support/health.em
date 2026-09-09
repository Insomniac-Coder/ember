## The declaring module. `pub(read)` makes `value` readable anywhere and
## writable only here; `pub` makes `max` both.
pub struct Health:
    pub(read) value: i32
    pub max: i32

pub fn make(v: i32) -> Health:
    return Health(v, 100)

## Writing a `pub(read)` field from the declaring module is allowed.
pub fn heal(mut h: Health, by: i32):
    h.value = h.value + by
