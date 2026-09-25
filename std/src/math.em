## `std.math` — Part XV's maths functions. Each one takes any number type.
##
## How the file fits together:
##
## - `Float` is the two decimal types, `f32` and `f64` (the two `extend`
##   lines below it). Its methods — `sqrt`, `sin`, and so on — are built into
##   those two types: `sqrt` of an `f64` is C's `sqrt`, of an `f32` C's `sqrtf`.
## - `Number` is every number type. Each `extend … implements Number` block adds
##   one type and says which decimal type its answers come back as (`type Real`)
##   and how to turn it into that type (`to_real`). Integers answer in `f64`, so
##   `sqrt(9)` is `3.0`; `f32` answers in `f32`, and `f64` in `f64`.
## - Each function turns its number into that decimal type and uses the `Float`
##   method: `sqrt(x)` is `x.to_real().sqrt()`, and its answer's type is
##   `T.Real`, the `Real` of whatever type `x` is.
##
## `min`, `max`, `clamp` and `abs` are built into the language for every number
## type, so they are not here. Nothing in this file uses heap memory (`[STD-1]`).

## The decimal types, and what each can do (`[STD-20]`, `[STD-27]`).
pub interface Float:
    fn abs(self) -> Self
    fn sqrt(self) -> Self
    fn cbrt(self) -> Self
    fn exp(self) -> Self
    fn exp2(self) -> Self
    fn ln(self) -> Self
    fn log2(self) -> Self
    fn log10(self) -> Self
    fn sin(self) -> Self
    fn cos(self) -> Self
    fn tan(self) -> Self
    fn asin(self) -> Self
    fn acos(self) -> Self
    fn atan(self) -> Self
    fn sinh(self) -> Self
    fn cosh(self) -> Self
    fn tanh(self) -> Self
    fn floor(self) -> Self
    fn ceil(self) -> Self
    fn trunc(self) -> Self
    fn fract(self) -> Self
    fn round(self) -> Self
    fn round_half_away(self) -> Self
    fn atan2(self, x: Self) -> Self
    fn hypot(self, y: Self) -> Self
    fn pow(self, e: Self) -> Self
    fn copysign(self, sign: Self) -> Self
    fn mul_add(self, a: Self, b: Self) -> Self
    fn is_nan(self) -> bool
    fn is_finite(self) -> bool
    fn is_infinite(self) -> bool

extend f32 implements Float:
    pass

extend f64 implements Float:
    pass

## Every number type: the decimal type its answers come back as, and how to
## turn it into one. (`i128` and `u128` join once they reach C, D-272.)
pub interface Number:
    type Real: Float
    fn to_real(self) -> Real

extend i8 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend i16 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend i32 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend i64 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend isize implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend u8 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend u16 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend u32 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend u64 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend usize implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend f32 implements Number:
    type Real = f32
    fn to_real(self) -> f32:
        return self

extend f64 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self

## `PI`, `TAU` (a full turn, 2π) and `E`. No type is written, so each takes the
## decimal type of wherever it is used: `x: f32 = PI` is `PI` as an `f32`, and
## anywhere else it is an `f64` (V.7).
pub const PI = 3.14159265358979323846264338327950288
pub const TAU = 6.28318530717958647692528676655900577
pub const E = 2.71828182845904523536028747135266250

## The functions (`[STD-21]`).
pub fn sqrt[T: Number](x: T) -> T.Real:
    return x.to_real().sqrt()

pub fn cbrt[T: Number](x: T) -> T.Real:
    return x.to_real().cbrt()

pub fn exp[T: Number](x: T) -> T.Real:
    return x.to_real().exp()

pub fn exp2[T: Number](x: T) -> T.Real:
    return x.to_real().exp2()

pub fn ln[T: Number](x: T) -> T.Real:
    return x.to_real().ln()

pub fn log2[T: Number](x: T) -> T.Real:
    return x.to_real().log2()

pub fn log10[T: Number](x: T) -> T.Real:
    return x.to_real().log10()

pub fn sin[T: Number](x: T) -> T.Real:
    return x.to_real().sin()

pub fn cos[T: Number](x: T) -> T.Real:
    return x.to_real().cos()

pub fn tan[T: Number](x: T) -> T.Real:
    return x.to_real().tan()

pub fn asin[T: Number](x: T) -> T.Real:
    return x.to_real().asin()

pub fn acos[T: Number](x: T) -> T.Real:
    return x.to_real().acos()

pub fn atan[T: Number](x: T) -> T.Real:
    return x.to_real().atan()

pub fn sinh[T: Number](x: T) -> T.Real:
    return x.to_real().sinh()

pub fn cosh[T: Number](x: T) -> T.Real:
    return x.to_real().cosh()

pub fn tanh[T: Number](x: T) -> T.Real:
    return x.to_real().tanh()

## The angle of the point (`x`, `y`).
pub fn atan2[T: Number](y: T, x: T) -> T.Real:
    return y.to_real().atan2(x.to_real())

## `x` to the power `e`.
pub fn pow[T: Number](x: T, e: T) -> T.Real:
    return x.to_real().pow(e.to_real())

## The long side of a right triangle: √(x² + y²).
pub fn hypot[T: Number](x: T, y: T) -> T.Real:
    return x.to_real().hypot(y.to_real())

## `1 / sqrt(x)`.
pub fn rsqrt[T: Number](x: T) -> T.Real:
    return 1.0 / x.to_real().sqrt()

## The point a fraction `t` of the way from `a` to `b`.
pub fn lerp[T: Number](a: T, b: T, t: T) -> T.Real:
    start = a.to_real()
    return start + (b.to_real() - start) * t.to_real()

## `0` below `edge0`, `1` above `edge1`, and a smooth curve between.
pub fn smoothstep[T: Number](edge0: T, edge1: T, x: T) -> T.Real:
    low = edge0.to_real()
    t = clamp((x.to_real() - low) / (edge1.to_real() - low), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)
