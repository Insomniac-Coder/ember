#! module overflow(wrap)

pub struct OverflowBox[T: Copy]:
    pub tag: T
    pub value: i8 = 127i8 + 1i8
