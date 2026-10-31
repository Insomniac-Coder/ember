#! module overflow(wrap)

[T: needs Copy]
pub struct OverflowBox:
    pub tag: T
    pub value: i8 = 127i8 + 1i8
