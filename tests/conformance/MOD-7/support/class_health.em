## Class visibility companion for MOD-7. The field is readable everywhere,
## but writing it requires the declaring module's class boundary.
pub open class ClassHealth:
    pub(read) value: i32

pub fn make(v: i32) -> ClassHealth:
    return ClassHealth(v)

pub fn heal(mut h: ClassHealth, by: i32):
    h.value = h.value + by
