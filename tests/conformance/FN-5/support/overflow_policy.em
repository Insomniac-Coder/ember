#! module overflow(wrap)

pub fn defaulted(x: i8, value: i8 = x + 1i8) -> i8:
    return value

[T]
pub fn generic(tag: T, value: i8) -> i8:
    return value + 1i8

pub fn callback() -> fn(i8) -> i8:
    return fn(value: i8) => value + 1i8
