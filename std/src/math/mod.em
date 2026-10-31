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
##   `sqrt(9)` is `3.0`; `f16` and `f32` answer in `f32`, and `f64` in `f64`.
## - Each function turns its number into that decimal type and uses the `Float`
##   method: `sqrt(x)` is `x.to_real().sqrt()`, and its answer's type is
##   `T.Real`, the `Real` of whatever type `x` is.
##
## `min`, `max`, `clamp` and `abs` are built into the language for every number
## type, so they are not here. Nothing in this file uses heap memory (`[STD-1]`).

## The decimal types, and what each can do (`[STD-20]`, `[STD-27]`).
pub interface Float:
    fn abs() -> Self
    fn sqrt() -> Self
    fn cbrt() -> Self
    fn exp() -> Self
    fn exp2() -> Self
    fn ln() -> Self
    fn log2() -> Self
    fn log10() -> Self
    fn sin() -> Self
    fn cos() -> Self
    fn tan() -> Self
    fn asin() -> Self
    fn acos() -> Self
    fn atan() -> Self
    fn sinh() -> Self
    fn cosh() -> Self
    fn tanh() -> Self
    fn floor() -> Self
    fn ceil() -> Self
    fn trunc() -> Self
    fn fract() -> Self
    fn round() -> Self
    fn round_half_away() -> Self
    fn atan2(x: Self) -> Self
    fn hypot(y: Self) -> Self
    fn pow(e: Self) -> Self
    fn copysign(sign: Self) -> Self
    fn mul_add(a: Self, b: Self) -> Self
    fn is_nan() -> bool
    fn is_finite() -> bool
    fn is_infinite() -> bool

extend f32 implements Float:
    pass

extend f64 implements Float:
    pass

## Every number type: the decimal type its answers come back as, and how to
## turn it into one.
pub interface Number:
    type Real: needs Float
    fn to_real() -> Real

extend i8 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend i16 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend i32 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend i64 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend i128 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend isize implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend u8 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend u16 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend u32 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend u64 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend u128 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend usize implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

## `f16` is not a `Float` (C has no maths functions for it); its answers come
## back as `f32`, which holds every `f16` exactly (ODR-041).
extend f16 implements Number:
    type Real = f32
    fn to_real() -> f32:
        return this as f32

extend f32 implements Number:
    type Real = f32
    fn to_real() -> f32:
        return this

extend f64 implements Number:
    type Real = f64
    fn to_real() -> f64:
        return this

## `PI`, `TAU` (a full turn, 2π) and `E`. No type is written, so each takes the
## decimal type of wherever it is used: `x: f32 = PI` is `PI` as an `f32`, and
## anywhere else it is an `f64` (V.7).
pub const PI = 3.14159265358979323846264338327950288
pub const TAU = 6.28318530717958647692528676655900577
pub const E = 2.71828182845904523536028747135266250

## The functions (`[STD-21]`).
[T: needs Number]
pub fn sqrt(x: T) -> T.Real:
    return x.to_real().sqrt()

[T: needs Number]
pub fn cbrt(x: T) -> T.Real:
    return x.to_real().cbrt()

[T: needs Number]
pub fn exp(x: T) -> T.Real:
    return x.to_real().exp()

[T: needs Number]
pub fn exp2(x: T) -> T.Real:
    return x.to_real().exp2()

[T: needs Number]
pub fn ln(x: T) -> T.Real:
    return x.to_real().ln()

[T: needs Number]
pub fn log2(x: T) -> T.Real:
    return x.to_real().log2()

[T: needs Number]
pub fn log10(x: T) -> T.Real:
    return x.to_real().log10()

[T: needs Number]
pub fn sin(x: T) -> T.Real:
    return x.to_real().sin()

[T: needs Number]
pub fn cos(x: T) -> T.Real:
    return x.to_real().cos()

[T: needs Number]
pub fn tan(x: T) -> T.Real:
    return x.to_real().tan()

[T: needs Number]
pub fn asin(x: T) -> T.Real:
    return x.to_real().asin()

[T: needs Number]
pub fn acos(x: T) -> T.Real:
    return x.to_real().acos()

[T: needs Number]
pub fn atan(x: T) -> T.Real:
    return x.to_real().atan()

[T: needs Number]
pub fn sinh(x: T) -> T.Real:
    return x.to_real().sinh()

[T: needs Number]
pub fn cosh(x: T) -> T.Real:
    return x.to_real().cosh()

[T: needs Number]
pub fn tanh(x: T) -> T.Real:
    return x.to_real().tanh()

## The angle of the point (`x`, `y`).
[T: needs Number]
pub fn atan2(y: T, x: T) -> T.Real:
    return y.to_real().atan2(x.to_real())

## `x` to the power `e`.
[T: needs Number]
pub fn pow(x: T, e: T) -> T.Real:
    return x.to_real().pow(e.to_real())

## The long side of a right triangle: √(x² + y²).
[T: needs Number]
pub fn hypot(x: T, y: T) -> T.Real:
    return x.to_real().hypot(y.to_real())

## `a * b + c` rounded once, not twice: a fused multiply-add, the same on
## every target (`[STD-3]`). It is `a.mul_add(b, c)`, C's `fma`.
[T: needs Number]
pub fn fma(a: T, b: T, c: T) -> T.Real:
    return a.to_real().mul_add(b.to_real(), c.to_real())

## `1 / sqrt(x)`.
[T: needs Number]
pub fn rsqrt(x: T) -> T.Real:
    return 1.0 / x.to_real().sqrt()

## The point a fraction `t` of the way from `a` to `b`.
[T: needs Number]
pub fn lerp(a: T, b: T, t: T) -> T.Real:
    start = a.to_real()
    return start + (b.to_real() - start) * t.to_real()

## `0` below `edge0`, `1` above `edge1`, and a smooth curve between.
[T: needs Number]
pub fn smoothstep(edge0: T, edge1: T, x: T) -> T.Real:
    low = edge0.to_real()
    t = clamp((x.to_real() - low) / (edge1.to_real() - low), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)

## -- vectors (`[STD-28]`, ODR-043) --------------------------------------------
##
## The operators work component by component, and a scalar on either side acts
## on every component: `v * 2.0`, `2.0 * v`. An integer vector's `//` and `%`
## are floor division and modulo, and its overflow panics as a component's
## would (`[TYP-8]`).
## (Written by `tools/gen_math_vectors.py`, to the matrices: edit the script, not this.)

## 2 `f32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct Vec2:
    pub x: f32
    pub y: f32

    pub const ZERO: Vec2 = Vec2(0.0, 0.0)
    pub const ONE: Vec2 = Vec2(1.0, 1.0)
    pub const X: Vec2 = Vec2(1.0, 0.0)
    pub const Y: Vec2 = Vec2(0.0, 1.0)

    ## Every component `v`.
    pub static fn splat(v: f32) -> Vec2:
        return Vec2(v, v)

    pub fn dot(o: Vec2) -> f32:
        return this.x * o.x + this.y * o.y

    ## The smaller of each pair of components.
    pub fn min(o: Vec2) -> Vec2:
        return Vec2(min(this.x, o.x), min(this.y, o.y))

    ## The larger of each pair of components.
    pub fn max(o: Vec2) -> Vec2:
        return Vec2(max(this.x, o.x), max(this.y, o.y))

    pub fn abs() -> Vec2:
        return Vec2(abs(this.x), abs(this.y))

    pub fn length_squared() -> f32:
        return this.dot(this)

    pub fn length() -> f32:
        return this.length_squared().sqrt()

    pub fn distance(o: Vec2) -> f32:
        return (this - o).length()

    pub fn distance_squared(o: Vec2) -> f32:
        return (this - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize() -> Vec2:
        length = this.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return this / length

    pub fn normalize_or_zero() -> Vec2:
        length = this.length()
        if length == 0.0:
            return Vec2.ZERO
        return this / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(a: Vec2, b: Vec2) -> Vec2:
        return Vec2(this.x.mul_add(a.x, b.x), this.y.mul_add(a.y, b.y))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(o: Vec2, t: f32) -> Vec2:
        return this + (o - this) * t

    ## This vector with a z component.
    pub fn extend(z: f32) -> Vec3:
        return Vec3(this.x, this.y, z)

extend Vec2 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec2

    fn add(o: Vec2) -> Vec2:
        return Vec2(this.x + o.x, this.y + o.y)

    fn sub(o: Vec2) -> Vec2:
        return Vec2(this.x - o.x, this.y - o.y)

    fn mul(o: Vec2) -> Vec2:
        return Vec2(this.x * o.x, this.y * o.y)

    fn div(o: Vec2) -> Vec2:
        return Vec2(this.x / o.x, this.y / o.y)

    fn neg() -> Vec2:
        return Vec2(-this.x, -this.y)

extend Vec2 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec2

    fn add(s: f32) -> Vec2:
        return Vec2(this.x + s, this.y + s)

    fn sub(s: f32) -> Vec2:
        return Vec2(this.x - s, this.y - s)

    fn mul(s: f32) -> Vec2:
        return Vec2(this.x * s, this.y * s)

    fn div(s: f32) -> Vec2:
        return Vec2(this.x / s, this.y / s)

extend f32 implements Add[Vec2], Sub[Vec2], Mul[Vec2], Div[Vec2]:
    type Output = Vec2

    fn add(v: Vec2) -> Vec2:
        return Vec2(this + v.x, this + v.y)

    fn sub(v: Vec2) -> Vec2:
        return Vec2(this - v.x, this - v.y)

    fn mul(v: Vec2) -> Vec2:
        return Vec2(this * v.x, this * v.y)

    fn div(v: Vec2) -> Vec2:
        return Vec2(this / v.x, this / v.y)

## 3 `f32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct Vec3:
    pub x: f32
    pub y: f32
    pub z: f32

    pub const ZERO: Vec3 = Vec3(0.0, 0.0, 0.0)
    pub const ONE: Vec3 = Vec3(1.0, 1.0, 1.0)
    pub const X: Vec3 = Vec3(1.0, 0.0, 0.0)
    pub const Y: Vec3 = Vec3(0.0, 1.0, 0.0)
    pub const Z: Vec3 = Vec3(0.0, 0.0, 1.0)

    ## Every component `v`.
    pub static fn splat(v: f32) -> Vec3:
        return Vec3(v, v, v)

    pub fn dot(o: Vec3) -> f32:
        return this.x * o.x + this.y * o.y + this.z * o.z

    ## The smaller of each pair of components.
    pub fn min(o: Vec3) -> Vec3:
        return Vec3(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z))

    ## The larger of each pair of components.
    pub fn max(o: Vec3) -> Vec3:
        return Vec3(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z))

    pub fn abs() -> Vec3:
        return Vec3(abs(this.x), abs(this.y), abs(this.z))

    pub fn length_squared() -> f32:
        return this.dot(this)

    pub fn length() -> f32:
        return this.length_squared().sqrt()

    pub fn distance(o: Vec3) -> f32:
        return (this - o).length()

    pub fn distance_squared(o: Vec3) -> f32:
        return (this - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize() -> Vec3:
        length = this.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return this / length

    pub fn normalize_or_zero() -> Vec3:
        length = this.length()
        if length == 0.0:
            return Vec3.ZERO
        return this / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(a: Vec3, b: Vec3) -> Vec3:
        return Vec3(this.x.mul_add(a.x, b.x), this.y.mul_add(a.y, b.y), this.z.mul_add(a.z, b.z))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(o: Vec3, t: f32) -> Vec3:
        return this + (o - this) * t

    ## The vector at right angles to both, by the right-hand rule.
    pub fn cross(o: Vec3) -> Vec3:
        return Vec3(this.y * o.z - this.z * o.y, this.z * o.x - this.x * o.z, this.x * o.y - this.y * o.x)

    ## This vector with a w component.
    pub fn extend(w: f32) -> Vec4:
        return Vec4(this.x, this.y, this.z, w)

    ## This vector without its last component.
    pub fn truncate() -> Vec2:
        return Vec2(this.x, this.y)

extend Vec3 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec3

    fn add(o: Vec3) -> Vec3:
        return Vec3(this.x + o.x, this.y + o.y, this.z + o.z)

    fn sub(o: Vec3) -> Vec3:
        return Vec3(this.x - o.x, this.y - o.y, this.z - o.z)

    fn mul(o: Vec3) -> Vec3:
        return Vec3(this.x * o.x, this.y * o.y, this.z * o.z)

    fn div(o: Vec3) -> Vec3:
        return Vec3(this.x / o.x, this.y / o.y, this.z / o.z)

    fn neg() -> Vec3:
        return Vec3(-this.x, -this.y, -this.z)

extend Vec3 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec3

    fn add(s: f32) -> Vec3:
        return Vec3(this.x + s, this.y + s, this.z + s)

    fn sub(s: f32) -> Vec3:
        return Vec3(this.x - s, this.y - s, this.z - s)

    fn mul(s: f32) -> Vec3:
        return Vec3(this.x * s, this.y * s, this.z * s)

    fn div(s: f32) -> Vec3:
        return Vec3(this.x / s, this.y / s, this.z / s)

extend f32 implements Add[Vec3], Sub[Vec3], Mul[Vec3], Div[Vec3]:
    type Output = Vec3

    fn add(v: Vec3) -> Vec3:
        return Vec3(this + v.x, this + v.y, this + v.z)

    fn sub(v: Vec3) -> Vec3:
        return Vec3(this - v.x, this - v.y, this - v.z)

    fn mul(v: Vec3) -> Vec3:
        return Vec3(this * v.x, this * v.y, this * v.z)

    fn div(v: Vec3) -> Vec3:
        return Vec3(this / v.x, this / v.y, this / v.z)

## 4 `f32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct Vec4:
    pub x: f32
    pub y: f32
    pub z: f32
    pub w: f32

    pub const ZERO: Vec4 = Vec4(0.0, 0.0, 0.0, 0.0)
    pub const ONE: Vec4 = Vec4(1.0, 1.0, 1.0, 1.0)
    pub const X: Vec4 = Vec4(1.0, 0.0, 0.0, 0.0)
    pub const Y: Vec4 = Vec4(0.0, 1.0, 0.0, 0.0)
    pub const Z: Vec4 = Vec4(0.0, 0.0, 1.0, 0.0)
    pub const W: Vec4 = Vec4(0.0, 0.0, 0.0, 1.0)

    ## Every component `v`.
    pub static fn splat(v: f32) -> Vec4:
        return Vec4(v, v, v, v)

    pub fn dot(o: Vec4) -> f32:
        return this.x * o.x + this.y * o.y + this.z * o.z + this.w * o.w

    ## The smaller of each pair of components.
    pub fn min(o: Vec4) -> Vec4:
        return Vec4(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z), min(this.w, o.w))

    ## The larger of each pair of components.
    pub fn max(o: Vec4) -> Vec4:
        return Vec4(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z), max(this.w, o.w))

    pub fn abs() -> Vec4:
        return Vec4(abs(this.x), abs(this.y), abs(this.z), abs(this.w))

    pub fn length_squared() -> f32:
        return this.dot(this)

    pub fn length() -> f32:
        return this.length_squared().sqrt()

    pub fn distance(o: Vec4) -> f32:
        return (this - o).length()

    pub fn distance_squared(o: Vec4) -> f32:
        return (this - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize() -> Vec4:
        length = this.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return this / length

    pub fn normalize_or_zero() -> Vec4:
        length = this.length()
        if length == 0.0:
            return Vec4.ZERO
        return this / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(a: Vec4, b: Vec4) -> Vec4:
        return Vec4(this.x.mul_add(a.x, b.x), this.y.mul_add(a.y, b.y), this.z.mul_add(a.z, b.z), this.w.mul_add(a.w, b.w))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(o: Vec4, t: f32) -> Vec4:
        return this + (o - this) * t

    ## This vector without its last component.
    pub fn truncate() -> Vec3:
        return Vec3(this.x, this.y, this.z)

extend Vec4 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec4

    fn add(o: Vec4) -> Vec4:
        return Vec4(this.x + o.x, this.y + o.y, this.z + o.z, this.w + o.w)

    fn sub(o: Vec4) -> Vec4:
        return Vec4(this.x - o.x, this.y - o.y, this.z - o.z, this.w - o.w)

    fn mul(o: Vec4) -> Vec4:
        return Vec4(this.x * o.x, this.y * o.y, this.z * o.z, this.w * o.w)

    fn div(o: Vec4) -> Vec4:
        return Vec4(this.x / o.x, this.y / o.y, this.z / o.z, this.w / o.w)

    fn neg() -> Vec4:
        return Vec4(-this.x, -this.y, -this.z, -this.w)

extend Vec4 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec4

    fn add(s: f32) -> Vec4:
        return Vec4(this.x + s, this.y + s, this.z + s, this.w + s)

    fn sub(s: f32) -> Vec4:
        return Vec4(this.x - s, this.y - s, this.z - s, this.w - s)

    fn mul(s: f32) -> Vec4:
        return Vec4(this.x * s, this.y * s, this.z * s, this.w * s)

    fn div(s: f32) -> Vec4:
        return Vec4(this.x / s, this.y / s, this.z / s, this.w / s)

extend f32 implements Add[Vec4], Sub[Vec4], Mul[Vec4], Div[Vec4]:
    type Output = Vec4

    fn add(v: Vec4) -> Vec4:
        return Vec4(this + v.x, this + v.y, this + v.z, this + v.w)

    fn sub(v: Vec4) -> Vec4:
        return Vec4(this - v.x, this - v.y, this - v.z, this - v.w)

    fn mul(v: Vec4) -> Vec4:
        return Vec4(this * v.x, this * v.y, this * v.z, this * v.w)

    fn div(v: Vec4) -> Vec4:
        return Vec4(this / v.x, this / v.y, this / v.z, this / v.w)

## 2 `i32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct IVec2:
    pub x: i32
    pub y: i32

    pub const ZERO: IVec2 = IVec2(0, 0)
    pub const ONE: IVec2 = IVec2(1, 1)
    pub const X: IVec2 = IVec2(1, 0)
    pub const Y: IVec2 = IVec2(0, 1)

    ## Every component `v`.
    pub static fn splat(v: i32) -> IVec2:
        return IVec2(v, v)

    pub fn dot(o: IVec2) -> i32:
        return this.x * o.x + this.y * o.y

    ## The smaller of each pair of components.
    pub fn min(o: IVec2) -> IVec2:
        return IVec2(min(this.x, o.x), min(this.y, o.y))

    ## The larger of each pair of components.
    pub fn max(o: IVec2) -> IVec2:
        return IVec2(max(this.x, o.x), max(this.y, o.y))

    pub fn abs() -> IVec2:
        return IVec2(abs(this.x), abs(this.y))

    ## This vector with a z component.
    pub fn extend(z: i32) -> IVec3:
        return IVec3(this.x, this.y, z)

extend IVec2 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec2

    fn add(o: IVec2) -> IVec2:
        return IVec2(this.x + o.x, this.y + o.y)

    fn sub(o: IVec2) -> IVec2:
        return IVec2(this.x - o.x, this.y - o.y)

    fn mul(o: IVec2) -> IVec2:
        return IVec2(this.x * o.x, this.y * o.y)

    fn floordiv(o: IVec2) -> IVec2:
        return IVec2(this.x // o.x, this.y // o.y)

    fn rem(o: IVec2) -> IVec2:
        return IVec2(this.x % o.x, this.y % o.y)

    fn neg() -> IVec2:
        return IVec2(-this.x, -this.y)

extend IVec2 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec2

    fn add(s: i32) -> IVec2:
        return IVec2(this.x + s, this.y + s)

    fn sub(s: i32) -> IVec2:
        return IVec2(this.x - s, this.y - s)

    fn mul(s: i32) -> IVec2:
        return IVec2(this.x * s, this.y * s)

    fn floordiv(s: i32) -> IVec2:
        return IVec2(this.x // s, this.y // s)

    fn rem(s: i32) -> IVec2:
        return IVec2(this.x % s, this.y % s)

extend i32 implements Add[IVec2], Sub[IVec2], Mul[IVec2], FloorDiv[IVec2], Rem[IVec2]:
    type Output = IVec2

    fn add(v: IVec2) -> IVec2:
        return IVec2(this + v.x, this + v.y)

    fn sub(v: IVec2) -> IVec2:
        return IVec2(this - v.x, this - v.y)

    fn mul(v: IVec2) -> IVec2:
        return IVec2(this * v.x, this * v.y)

    fn floordiv(v: IVec2) -> IVec2:
        return IVec2(this // v.x, this // v.y)

    fn rem(v: IVec2) -> IVec2:
        return IVec2(this % v.x, this % v.y)

## 3 `i32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct IVec3:
    pub x: i32
    pub y: i32
    pub z: i32

    pub const ZERO: IVec3 = IVec3(0, 0, 0)
    pub const ONE: IVec3 = IVec3(1, 1, 1)
    pub const X: IVec3 = IVec3(1, 0, 0)
    pub const Y: IVec3 = IVec3(0, 1, 0)
    pub const Z: IVec3 = IVec3(0, 0, 1)

    ## Every component `v`.
    pub static fn splat(v: i32) -> IVec3:
        return IVec3(v, v, v)

    pub fn dot(o: IVec3) -> i32:
        return this.x * o.x + this.y * o.y + this.z * o.z

    ## The smaller of each pair of components.
    pub fn min(o: IVec3) -> IVec3:
        return IVec3(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z))

    ## The larger of each pair of components.
    pub fn max(o: IVec3) -> IVec3:
        return IVec3(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z))

    pub fn abs() -> IVec3:
        return IVec3(abs(this.x), abs(this.y), abs(this.z))

    ## This vector with a w component.
    pub fn extend(w: i32) -> IVec4:
        return IVec4(this.x, this.y, this.z, w)

    ## This vector without its last component.
    pub fn truncate() -> IVec2:
        return IVec2(this.x, this.y)

extend IVec3 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec3

    fn add(o: IVec3) -> IVec3:
        return IVec3(this.x + o.x, this.y + o.y, this.z + o.z)

    fn sub(o: IVec3) -> IVec3:
        return IVec3(this.x - o.x, this.y - o.y, this.z - o.z)

    fn mul(o: IVec3) -> IVec3:
        return IVec3(this.x * o.x, this.y * o.y, this.z * o.z)

    fn floordiv(o: IVec3) -> IVec3:
        return IVec3(this.x // o.x, this.y // o.y, this.z // o.z)

    fn rem(o: IVec3) -> IVec3:
        return IVec3(this.x % o.x, this.y % o.y, this.z % o.z)

    fn neg() -> IVec3:
        return IVec3(-this.x, -this.y, -this.z)

extend IVec3 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec3

    fn add(s: i32) -> IVec3:
        return IVec3(this.x + s, this.y + s, this.z + s)

    fn sub(s: i32) -> IVec3:
        return IVec3(this.x - s, this.y - s, this.z - s)

    fn mul(s: i32) -> IVec3:
        return IVec3(this.x * s, this.y * s, this.z * s)

    fn floordiv(s: i32) -> IVec3:
        return IVec3(this.x // s, this.y // s, this.z // s)

    fn rem(s: i32) -> IVec3:
        return IVec3(this.x % s, this.y % s, this.z % s)

extend i32 implements Add[IVec3], Sub[IVec3], Mul[IVec3], FloorDiv[IVec3], Rem[IVec3]:
    type Output = IVec3

    fn add(v: IVec3) -> IVec3:
        return IVec3(this + v.x, this + v.y, this + v.z)

    fn sub(v: IVec3) -> IVec3:
        return IVec3(this - v.x, this - v.y, this - v.z)

    fn mul(v: IVec3) -> IVec3:
        return IVec3(this * v.x, this * v.y, this * v.z)

    fn floordiv(v: IVec3) -> IVec3:
        return IVec3(this // v.x, this // v.y, this // v.z)

    fn rem(v: IVec3) -> IVec3:
        return IVec3(this % v.x, this % v.y, this % v.z)

## 4 `i32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct IVec4:
    pub x: i32
    pub y: i32
    pub z: i32
    pub w: i32

    pub const ZERO: IVec4 = IVec4(0, 0, 0, 0)
    pub const ONE: IVec4 = IVec4(1, 1, 1, 1)
    pub const X: IVec4 = IVec4(1, 0, 0, 0)
    pub const Y: IVec4 = IVec4(0, 1, 0, 0)
    pub const Z: IVec4 = IVec4(0, 0, 1, 0)
    pub const W: IVec4 = IVec4(0, 0, 0, 1)

    ## Every component `v`.
    pub static fn splat(v: i32) -> IVec4:
        return IVec4(v, v, v, v)

    pub fn dot(o: IVec4) -> i32:
        return this.x * o.x + this.y * o.y + this.z * o.z + this.w * o.w

    ## The smaller of each pair of components.
    pub fn min(o: IVec4) -> IVec4:
        return IVec4(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z), min(this.w, o.w))

    ## The larger of each pair of components.
    pub fn max(o: IVec4) -> IVec4:
        return IVec4(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z), max(this.w, o.w))

    pub fn abs() -> IVec4:
        return IVec4(abs(this.x), abs(this.y), abs(this.z), abs(this.w))

    ## This vector without its last component.
    pub fn truncate() -> IVec3:
        return IVec3(this.x, this.y, this.z)

extend IVec4 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec4

    fn add(o: IVec4) -> IVec4:
        return IVec4(this.x + o.x, this.y + o.y, this.z + o.z, this.w + o.w)

    fn sub(o: IVec4) -> IVec4:
        return IVec4(this.x - o.x, this.y - o.y, this.z - o.z, this.w - o.w)

    fn mul(o: IVec4) -> IVec4:
        return IVec4(this.x * o.x, this.y * o.y, this.z * o.z, this.w * o.w)

    fn floordiv(o: IVec4) -> IVec4:
        return IVec4(this.x // o.x, this.y // o.y, this.z // o.z, this.w // o.w)

    fn rem(o: IVec4) -> IVec4:
        return IVec4(this.x % o.x, this.y % o.y, this.z % o.z, this.w % o.w)

    fn neg() -> IVec4:
        return IVec4(-this.x, -this.y, -this.z, -this.w)

extend IVec4 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec4

    fn add(s: i32) -> IVec4:
        return IVec4(this.x + s, this.y + s, this.z + s, this.w + s)

    fn sub(s: i32) -> IVec4:
        return IVec4(this.x - s, this.y - s, this.z - s, this.w - s)

    fn mul(s: i32) -> IVec4:
        return IVec4(this.x * s, this.y * s, this.z * s, this.w * s)

    fn floordiv(s: i32) -> IVec4:
        return IVec4(this.x // s, this.y // s, this.z // s, this.w // s)

    fn rem(s: i32) -> IVec4:
        return IVec4(this.x % s, this.y % s, this.z % s, this.w % s)

extend i32 implements Add[IVec4], Sub[IVec4], Mul[IVec4], FloorDiv[IVec4], Rem[IVec4]:
    type Output = IVec4

    fn add(v: IVec4) -> IVec4:
        return IVec4(this + v.x, this + v.y, this + v.z, this + v.w)

    fn sub(v: IVec4) -> IVec4:
        return IVec4(this - v.x, this - v.y, this - v.z, this - v.w)

    fn mul(v: IVec4) -> IVec4:
        return IVec4(this * v.x, this * v.y, this * v.z, this * v.w)

    fn floordiv(v: IVec4) -> IVec4:
        return IVec4(this // v.x, this // v.y, this // v.z, this // v.w)

    fn rem(v: IVec4) -> IVec4:
        return IVec4(this % v.x, this % v.y, this % v.z, this % v.w)

## 2 `u32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct UVec2:
    pub x: u32
    pub y: u32

    pub const ZERO: UVec2 = UVec2(0, 0)
    pub const ONE: UVec2 = UVec2(1, 1)
    pub const X: UVec2 = UVec2(1, 0)
    pub const Y: UVec2 = UVec2(0, 1)

    ## Every component `v`.
    pub static fn splat(v: u32) -> UVec2:
        return UVec2(v, v)

    pub fn dot(o: UVec2) -> u32:
        return this.x * o.x + this.y * o.y

    ## The smaller of each pair of components.
    pub fn min(o: UVec2) -> UVec2:
        return UVec2(min(this.x, o.x), min(this.y, o.y))

    ## The larger of each pair of components.
    pub fn max(o: UVec2) -> UVec2:
        return UVec2(max(this.x, o.x), max(this.y, o.y))

    ## This vector with a z component.
    pub fn extend(z: u32) -> UVec3:
        return UVec3(this.x, this.y, z)

extend UVec2 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec2

    fn add(o: UVec2) -> UVec2:
        return UVec2(this.x + o.x, this.y + o.y)

    fn sub(o: UVec2) -> UVec2:
        return UVec2(this.x - o.x, this.y - o.y)

    fn mul(o: UVec2) -> UVec2:
        return UVec2(this.x * o.x, this.y * o.y)

    fn floordiv(o: UVec2) -> UVec2:
        return UVec2(this.x // o.x, this.y // o.y)

    fn rem(o: UVec2) -> UVec2:
        return UVec2(this.x % o.x, this.y % o.y)

extend UVec2 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec2

    fn add(s: u32) -> UVec2:
        return UVec2(this.x + s, this.y + s)

    fn sub(s: u32) -> UVec2:
        return UVec2(this.x - s, this.y - s)

    fn mul(s: u32) -> UVec2:
        return UVec2(this.x * s, this.y * s)

    fn floordiv(s: u32) -> UVec2:
        return UVec2(this.x // s, this.y // s)

    fn rem(s: u32) -> UVec2:
        return UVec2(this.x % s, this.y % s)

extend u32 implements Add[UVec2], Sub[UVec2], Mul[UVec2], FloorDiv[UVec2], Rem[UVec2]:
    type Output = UVec2

    fn add(v: UVec2) -> UVec2:
        return UVec2(this + v.x, this + v.y)

    fn sub(v: UVec2) -> UVec2:
        return UVec2(this - v.x, this - v.y)

    fn mul(v: UVec2) -> UVec2:
        return UVec2(this * v.x, this * v.y)

    fn floordiv(v: UVec2) -> UVec2:
        return UVec2(this // v.x, this // v.y)

    fn rem(v: UVec2) -> UVec2:
        return UVec2(this % v.x, this % v.y)

## 3 `u32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct UVec3:
    pub x: u32
    pub y: u32
    pub z: u32

    pub const ZERO: UVec3 = UVec3(0, 0, 0)
    pub const ONE: UVec3 = UVec3(1, 1, 1)
    pub const X: UVec3 = UVec3(1, 0, 0)
    pub const Y: UVec3 = UVec3(0, 1, 0)
    pub const Z: UVec3 = UVec3(0, 0, 1)

    ## Every component `v`.
    pub static fn splat(v: u32) -> UVec3:
        return UVec3(v, v, v)

    pub fn dot(o: UVec3) -> u32:
        return this.x * o.x + this.y * o.y + this.z * o.z

    ## The smaller of each pair of components.
    pub fn min(o: UVec3) -> UVec3:
        return UVec3(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z))

    ## The larger of each pair of components.
    pub fn max(o: UVec3) -> UVec3:
        return UVec3(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z))

    ## This vector with a w component.
    pub fn extend(w: u32) -> UVec4:
        return UVec4(this.x, this.y, this.z, w)

    ## This vector without its last component.
    pub fn truncate() -> UVec2:
        return UVec2(this.x, this.y)

extend UVec3 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec3

    fn add(o: UVec3) -> UVec3:
        return UVec3(this.x + o.x, this.y + o.y, this.z + o.z)

    fn sub(o: UVec3) -> UVec3:
        return UVec3(this.x - o.x, this.y - o.y, this.z - o.z)

    fn mul(o: UVec3) -> UVec3:
        return UVec3(this.x * o.x, this.y * o.y, this.z * o.z)

    fn floordiv(o: UVec3) -> UVec3:
        return UVec3(this.x // o.x, this.y // o.y, this.z // o.z)

    fn rem(o: UVec3) -> UVec3:
        return UVec3(this.x % o.x, this.y % o.y, this.z % o.z)

extend UVec3 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec3

    fn add(s: u32) -> UVec3:
        return UVec3(this.x + s, this.y + s, this.z + s)

    fn sub(s: u32) -> UVec3:
        return UVec3(this.x - s, this.y - s, this.z - s)

    fn mul(s: u32) -> UVec3:
        return UVec3(this.x * s, this.y * s, this.z * s)

    fn floordiv(s: u32) -> UVec3:
        return UVec3(this.x // s, this.y // s, this.z // s)

    fn rem(s: u32) -> UVec3:
        return UVec3(this.x % s, this.y % s, this.z % s)

extend u32 implements Add[UVec3], Sub[UVec3], Mul[UVec3], FloorDiv[UVec3], Rem[UVec3]:
    type Output = UVec3

    fn add(v: UVec3) -> UVec3:
        return UVec3(this + v.x, this + v.y, this + v.z)

    fn sub(v: UVec3) -> UVec3:
        return UVec3(this - v.x, this - v.y, this - v.z)

    fn mul(v: UVec3) -> UVec3:
        return UVec3(this * v.x, this * v.y, this * v.z)

    fn floordiv(v: UVec3) -> UVec3:
        return UVec3(this // v.x, this // v.y, this // v.z)

    fn rem(v: UVec3) -> UVec3:
        return UVec3(this % v.x, this % v.y, this % v.z)

## 4 `u32`s (`[STD-28]`, ODR-043).
@derive(Copy)
@layout(c)
pub struct UVec4:
    pub x: u32
    pub y: u32
    pub z: u32
    pub w: u32

    pub const ZERO: UVec4 = UVec4(0, 0, 0, 0)
    pub const ONE: UVec4 = UVec4(1, 1, 1, 1)
    pub const X: UVec4 = UVec4(1, 0, 0, 0)
    pub const Y: UVec4 = UVec4(0, 1, 0, 0)
    pub const Z: UVec4 = UVec4(0, 0, 1, 0)
    pub const W: UVec4 = UVec4(0, 0, 0, 1)

    ## Every component `v`.
    pub static fn splat(v: u32) -> UVec4:
        return UVec4(v, v, v, v)

    pub fn dot(o: UVec4) -> u32:
        return this.x * o.x + this.y * o.y + this.z * o.z + this.w * o.w

    ## The smaller of each pair of components.
    pub fn min(o: UVec4) -> UVec4:
        return UVec4(min(this.x, o.x), min(this.y, o.y), min(this.z, o.z), min(this.w, o.w))

    ## The larger of each pair of components.
    pub fn max(o: UVec4) -> UVec4:
        return UVec4(max(this.x, o.x), max(this.y, o.y), max(this.z, o.z), max(this.w, o.w))

    ## This vector without its last component.
    pub fn truncate() -> UVec3:
        return UVec3(this.x, this.y, this.z)

extend UVec4 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec4

    fn add(o: UVec4) -> UVec4:
        return UVec4(this.x + o.x, this.y + o.y, this.z + o.z, this.w + o.w)

    fn sub(o: UVec4) -> UVec4:
        return UVec4(this.x - o.x, this.y - o.y, this.z - o.z, this.w - o.w)

    fn mul(o: UVec4) -> UVec4:
        return UVec4(this.x * o.x, this.y * o.y, this.z * o.z, this.w * o.w)

    fn floordiv(o: UVec4) -> UVec4:
        return UVec4(this.x // o.x, this.y // o.y, this.z // o.z, this.w // o.w)

    fn rem(o: UVec4) -> UVec4:
        return UVec4(this.x % o.x, this.y % o.y, this.z % o.z, this.w % o.w)

extend UVec4 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec4

    fn add(s: u32) -> UVec4:
        return UVec4(this.x + s, this.y + s, this.z + s, this.w + s)

    fn sub(s: u32) -> UVec4:
        return UVec4(this.x - s, this.y - s, this.z - s, this.w - s)

    fn mul(s: u32) -> UVec4:
        return UVec4(this.x * s, this.y * s, this.z * s, this.w * s)

    fn floordiv(s: u32) -> UVec4:
        return UVec4(this.x // s, this.y // s, this.z // s, this.w // s)

    fn rem(s: u32) -> UVec4:
        return UVec4(this.x % s, this.y % s, this.z % s, this.w % s)

extend u32 implements Add[UVec4], Sub[UVec4], Mul[UVec4], FloorDiv[UVec4], Rem[UVec4]:
    type Output = UVec4

    fn add(v: UVec4) -> UVec4:
        return UVec4(this + v.x, this + v.y, this + v.z, this + v.w)

    fn sub(v: UVec4) -> UVec4:
        return UVec4(this - v.x, this - v.y, this - v.z, this - v.w)

    fn mul(v: UVec4) -> UVec4:
        return UVec4(this * v.x, this * v.y, this * v.z, this * v.w)

    fn floordiv(v: UVec4) -> UVec4:
        return UVec4(this // v.x, this // v.y, this // v.z, this // v.w)

    fn rem(v: UVec4) -> UVec4:
        return UVec4(this % v.x, this % v.y, this % v.z, this % v.w)


## -- matrices (`[STD-28]`, ODR-043) -------------------------------------------
##
## Square matrices of `f32`s, column-major: `x_axis`, `y_axis`, … are the
## columns, so `m * v` is `m.x_axis * v.x + m.y_axis * v.y + …`, and a
## transform's translation is `Mat4.w_axis`. `m * n` applies `n` first. The
## products are fused multiply-adds (`[STD-3]`): each term after the first is
## added with one rounding.

## A 2×2 matrix: a rotation or scale in the plane.
@derive(Copy)
@layout(c)
pub struct Mat2:
    pub x_axis: Vec2
    pub y_axis: Vec2

    pub const ZERO: Mat2 = Mat2(Vec2(0.0, 0.0), Vec2(0.0, 0.0))
    pub const IDENTITY: Mat2 = Mat2(Vec2(1.0, 0.0), Vec2(0.0, 1.0))

    pub static fn from_cols(x_axis: Vec2, y_axis: Vec2) -> Mat2:
        return Mat2(x_axis, y_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub static fn from_diagonal(d: Vec2) -> Mat2:
        return Mat2(Vec2(d.x, 0.0), Vec2(0.0, d.y))

    ## A rotation by `angle` radians, counterclockwise.
    pub static fn from_angle(angle: f32) -> Mat2:
        s = angle.sin()
        c = angle.cos()
        return Mat2(Vec2(c, s), Vec2(-s, c))

    ## Column `i`, or `None` for an `i` other than 0 and 1 (`col`'s twin, `[ERR-13]`).
    pub fn try_col(i: int) -> Option[Vec2]:
        if i < 0 or i > 1:
            return None
        return Some(this.col(i))

    ## Column `i`, 0 or 1; another `i` panics (`try_col` gives an `Option`).
    pub fn col(i: int) -> Vec2:
        if i == 0:
            return this.x_axis
        if i == 1:
            return this.y_axis
        panic("a Mat2 has columns 0 and 1 only; use .try_col(i) for an Option")

    ## Row `i`, or `None` for an `i` other than 0 and 1 (`row`'s twin, `[ERR-13]`).
    pub fn try_row(i: int) -> Option[Vec2]:
        if i < 0 or i > 1:
            return None
        return Some(this.row(i))

    ## Row `i`, 0 or 1; another `i` panics (`try_row` gives an `Option`).
    pub fn row(i: int) -> Vec2:
        if i == 0:
            return Vec2(this.x_axis.x, this.y_axis.x)
        if i == 1:
            return Vec2(this.x_axis.y, this.y_axis.y)
        panic("a Mat2 has rows 0 and 1 only; use .try_row(i) for an Option")

    pub fn transpose() -> Mat2:
        return Mat2(this.row(0), this.row(1))

    pub fn determinant() -> f32:
        return this.x_axis.x * this.y_axis.y - this.y_axis.x * this.x_axis.y

    ## The inverse, or `None` for a matrix with none (its determinant zero).
    pub fn try_inverse() -> Option[Mat2]:
        det = this.determinant()
        if det == 0.0:
            return None
        inv = 1.0 / det
        return Some(Mat2(Vec2(this.y_axis.y * inv, -this.x_axis.y * inv), Vec2(-this.y_axis.x * inv, this.x_axis.x * inv)))

    ## The inverse; a matrix with none panics.
    pub fn inverse() -> Mat2:
        match this.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

extend Mat2 implements Add, Sub, Mul, Neg:
    type Output = Mat2

    fn add(o: Mat2) -> Mat2:
        return Mat2(this.x_axis + o.x_axis, this.y_axis + o.y_axis)

    fn sub(o: Mat2) -> Mat2:
        return Mat2(this.x_axis - o.x_axis, this.y_axis - o.y_axis)

    fn mul(o: Mat2) -> Mat2:
        return Mat2(this * o.x_axis, this * o.y_axis)

    fn neg() -> Mat2:
        return Mat2(-this.x_axis, -this.y_axis)

extend Mat2 implements Mul[Vec2]:
    type Output = Vec2

    fn mul(v: Vec2) -> Vec2:
        return this.y_axis.mul_add(Vec2.splat(v.y), this.x_axis * v.x)

extend Mat2 implements Mul[f32]:
    type Output = Mat2

    fn mul(s: f32) -> Mat2:
        return Mat2(this.x_axis * s, this.y_axis * s)

extend f32 implements Mul[Mat2]:
    type Output = Mat2

    fn mul(m: Mat2) -> Mat2:
        return m * this

## A 3×3 matrix: a rotation or scale in space.
@derive(Copy)
@layout(c)
pub struct Mat3:
    pub x_axis: Vec3
    pub y_axis: Vec3
    pub z_axis: Vec3

    pub const ZERO: Mat3 = Mat3(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 0.0))
    pub const IDENTITY: Mat3 = Mat3(Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0), Vec3(0.0, 0.0, 1.0))

    pub static fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Mat3:
        return Mat3(x_axis, y_axis, z_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub static fn from_diagonal(d: Vec3) -> Mat3:
        return Mat3(Vec3(d.x, 0.0, 0.0), Vec3(0.0, d.y, 0.0), Vec3(0.0, 0.0, d.z))

    ## A scale by `s` along each axis.
    pub static fn from_scale(s: Vec3) -> Mat3:
        return Mat3.from_diagonal(s)

    ## A rotation by `angle` radians about the z axis, counterclockwise
    ## looking down it.
    pub static fn from_angle(angle: f32) -> Mat3:
        s = angle.sin()
        c = angle.cos()
        return Mat3(Vec3(c, s, 0.0), Vec3(-s, c, 0.0), Vec3(0.0, 0.0, 1.0))

    ## The rotation `q` is, for a `q` of length one.
    pub static fn from_quat(q: Quat) -> Mat3:
        x2 = q.x + q.x
        y2 = q.y + q.y
        z2 = q.z + q.z
        xx = q.x * x2
        xy = q.x * y2
        xz = q.x * z2
        yy = q.y * y2
        yz = q.y * z2
        zz = q.z * z2
        wx = q.w * x2
        wy = q.w * y2
        wz = q.w * z2
        return Mat3(
            Vec3(1.0 - (yy + zz), xy + wz, xz - wy),
            Vec3(xy - wz, 1.0 - (xx + zz), yz + wx),
            Vec3(xz + wy, yz - wx, 1.0 - (xx + yy)),
        )

    ## Column `i`, or `None` for an `i` outside 0 to 2 (`col`'s twin, `[ERR-13]`).
    pub fn try_col(i: int) -> Option[Vec3]:
        if i < 0 or i > 2:
            return None
        return Some(this.col(i))

    ## Column `i`, 0 to 2; another `i` panics (`try_col` gives an `Option`).
    pub fn col(i: int) -> Vec3:
        if i == 0:
            return this.x_axis
        if i == 1:
            return this.y_axis
        if i == 2:
            return this.z_axis
        panic("a Mat3 has columns 0 to 2 only; use .try_col(i) for an Option")

    ## Row `i`, or `None` for an `i` outside 0 to 2 (`row`'s twin, `[ERR-13]`).
    pub fn try_row(i: int) -> Option[Vec3]:
        if i < 0 or i > 2:
            return None
        return Some(this.row(i))

    ## Row `i`, 0 to 2; another `i` panics (`try_row` gives an `Option`).
    pub fn row(i: int) -> Vec3:
        if i == 0:
            return Vec3(this.x_axis.x, this.y_axis.x, this.z_axis.x)
        if i == 1:
            return Vec3(this.x_axis.y, this.y_axis.y, this.z_axis.y)
        if i == 2:
            return Vec3(this.x_axis.z, this.y_axis.z, this.z_axis.z)
        panic("a Mat3 has rows 0 to 2 only; use .try_row(i) for an Option")

    pub fn transpose() -> Mat3:
        return Mat3(this.row(0), this.row(1), this.row(2))

    pub fn determinant() -> f32:
        return this.x_axis.dot(this.y_axis.cross(this.z_axis))

    ## The inverse, or `None` for a matrix with none (its determinant zero).
    ## Its rows are the cross products of the columns, over the determinant.
    pub fn try_inverse() -> Option[Mat3]:
        yz = this.y_axis.cross(this.z_axis)
        zx = this.z_axis.cross(this.x_axis)
        xy = this.x_axis.cross(this.y_axis)
        det = this.x_axis.dot(yz)
        if det == 0.0:
            return None
        return Some(Mat3(yz, zx, xy).transpose() * (1.0 / det))

    ## The inverse; a matrix with none panics.
    pub fn inverse() -> Mat3:
        match this.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

extend Mat3 implements Add, Sub, Mul, Neg:
    type Output = Mat3

    fn add(o: Mat3) -> Mat3:
        return Mat3(this.x_axis + o.x_axis, this.y_axis + o.y_axis, this.z_axis + o.z_axis)

    fn sub(o: Mat3) -> Mat3:
        return Mat3(this.x_axis - o.x_axis, this.y_axis - o.y_axis, this.z_axis - o.z_axis)

    fn mul(o: Mat3) -> Mat3:
        return Mat3(this * o.x_axis, this * o.y_axis, this * o.z_axis)

    fn neg() -> Mat3:
        return Mat3(-this.x_axis, -this.y_axis, -this.z_axis)

extend Mat3 implements Mul[Vec3]:
    type Output = Vec3

    fn mul(v: Vec3) -> Vec3:
        r = this.y_axis.mul_add(Vec3.splat(v.y), this.x_axis * v.x)
        return this.z_axis.mul_add(Vec3.splat(v.z), r)

extend Mat3 implements Mul[f32]:
    type Output = Mat3

    fn mul(s: f32) -> Mat3:
        return Mat3(this.x_axis * s, this.y_axis * s, this.z_axis * s)

extend f32 implements Mul[Mat3]:
    type Output = Mat3

    fn mul(m: Mat3) -> Mat3:
        return m * this

## A 4×4 matrix: a transform with translation, or a projection.
@derive(Copy)
@layout(c)
pub struct Mat4:
    pub x_axis: Vec4
    pub y_axis: Vec4
    pub z_axis: Vec4
    pub w_axis: Vec4

    pub const ZERO: Mat4 = Mat4(Vec4(0.0, 0.0, 0.0, 0.0), Vec4(0.0, 0.0, 0.0, 0.0), Vec4(0.0, 0.0, 0.0, 0.0), Vec4(0.0, 0.0, 0.0, 0.0))
    pub const IDENTITY: Mat4 = Mat4(Vec4(1.0, 0.0, 0.0, 0.0), Vec4(0.0, 1.0, 0.0, 0.0), Vec4(0.0, 0.0, 1.0, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    pub static fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Mat4:
        return Mat4(x_axis, y_axis, z_axis, w_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub static fn from_diagonal(d: Vec4) -> Mat4:
        return Mat4(Vec4(d.x, 0.0, 0.0, 0.0), Vec4(0.0, d.y, 0.0, 0.0), Vec4(0.0, 0.0, d.z, 0.0), Vec4(0.0, 0.0, 0.0, d.w))

    ## A move by `t`.
    pub static fn from_translation(t: Vec3) -> Mat4:
        return Mat4(Vec4(1.0, 0.0, 0.0, 0.0), Vec4(0.0, 1.0, 0.0, 0.0), Vec4(0.0, 0.0, 1.0, 0.0), t.extend(1.0))

    ## A scale by `s` along each axis.
    pub static fn from_scale(s: Vec3) -> Mat4:
        return Mat4.from_diagonal(s.extend(1.0))

    ## The rotation `q` is, for a `q` of length one.
    pub static fn from_quat(q: Quat) -> Mat4:
        m = Mat3.from_quat(q)
        return Mat4(m.x_axis.extend(0.0), m.y_axis.extend(0.0), m.z_axis.extend(0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the x axis.
    pub static fn from_rotation_x(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(1.0, 0.0, 0.0, 0.0), Vec4(0.0, c, s, 0.0), Vec4(0.0, -s, c, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the y axis.
    pub static fn from_rotation_y(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(c, 0.0, -s, 0.0), Vec4(0.0, 1.0, 0.0, 0.0), Vec4(s, 0.0, c, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the z axis.
    pub static fn from_rotation_z(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(c, s, 0.0, 0.0), Vec4(-s, c, 0.0, 0.0), Vec4(0.0, 0.0, 1.0, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## Scale by `scale`, then rotate by `rotation`, then move by
    ## `translation`.
    pub static fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Mat4:
        m = Mat3.from_quat(rotation)
        return Mat4(
            (m.x_axis * scale.x).extend(0.0),
            (m.y_axis * scale.y).extend(0.0),
            (m.z_axis * scale.z).extend(0.0),
            translation.extend(1.0),
        )

    ## A right-handed view from `eye` towards `center`, with `up` the rough
    ## direction of the view's y axis: the camera looks down its −z axis.
    pub static fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Mat4:
        f = (center - eye).normalize()
        s = f.cross(up).normalize()
        u = s.cross(f)
        return Mat4(
            Vec4(s.x, u.x, -f.x, 0.0),
            Vec4(s.y, u.y, -f.y, 0.0),
            Vec4(s.z, u.z, -f.z, 0.0),
            Vec4(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        )

    ## A right-handed perspective projection with depth in `[0, 1]`: `near`
    ## maps to 0 and `far` to 1. `fov_y` is the vertical field of view in
    ## radians, `aspect` the width over the height.
    pub static fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4:
        h = 1.0 / (fov_y * 0.5).tan()
        w = h / aspect
        r = far / (near - far)
        return Mat4(Vec4(w, 0.0, 0.0, 0.0), Vec4(0.0, h, 0.0, 0.0), Vec4(0.0, 0.0, r, -1.0), Vec4(0.0, 0.0, r * near, 0.0))

    ## A right-handed orthographic projection with depth in `[0, 1]`.
    pub static fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4:
        rw = 1.0 / (right - left)
        rh = 1.0 / (top - bottom)
        r = 1.0 / (near - far)
        return Mat4(
            Vec4(rw + rw, 0.0, 0.0, 0.0),
            Vec4(0.0, rh + rh, 0.0, 0.0),
            Vec4(0.0, 0.0, r, 0.0),
            Vec4(-(left + right) * rw, -(top + bottom) * rh, r * near, 1.0),
        )

    ## Column `i`, or `None` for an `i` outside 0 to 3 (`col`'s twin, `[ERR-13]`).
    pub fn try_col(i: int) -> Option[Vec4]:
        if i < 0 or i > 3:
            return None
        return Some(this.col(i))

    ## Column `i`, 0 to 3; another `i` panics (`try_col` gives an `Option`).
    pub fn col(i: int) -> Vec4:
        if i == 0:
            return this.x_axis
        if i == 1:
            return this.y_axis
        if i == 2:
            return this.z_axis
        if i == 3:
            return this.w_axis
        panic("a Mat4 has columns 0 to 3 only; use .try_col(i) for an Option")

    ## Row `i`, or `None` for an `i` outside 0 to 3 (`row`'s twin, `[ERR-13]`).
    pub fn try_row(i: int) -> Option[Vec4]:
        if i < 0 or i > 3:
            return None
        return Some(this.row(i))

    ## Row `i`, 0 to 3; another `i` panics (`try_row` gives an `Option`).
    pub fn row(i: int) -> Vec4:
        if i == 0:
            return Vec4(this.x_axis.x, this.y_axis.x, this.z_axis.x, this.w_axis.x)
        if i == 1:
            return Vec4(this.x_axis.y, this.y_axis.y, this.z_axis.y, this.w_axis.y)
        if i == 2:
            return Vec4(this.x_axis.z, this.y_axis.z, this.z_axis.z, this.w_axis.z)
        if i == 3:
            return Vec4(this.x_axis.w, this.y_axis.w, this.z_axis.w, this.w_axis.w)
        panic("a Mat4 has rows 0 to 3 only; use .try_row(i) for an Option")

    pub fn transpose() -> Mat4:
        return Mat4(this.row(0), this.row(1), this.row(2), this.row(3))

    pub fn determinant() -> f32:
        m = MinorPairs.of(this)
        return m.determinant()

    ## The inverse, or `None` for a matrix with none (its determinant zero):
    ## the adjugate over the determinant, from the 2×2 minors of the top and
    ## bottom halves.
    pub fn try_inverse() -> Option[Mat4]:
        m = MinorPairs.of(this)
        det = m.determinant()
        if det == 0.0:
            return None
        a = this.row(0)
        b = this.row(1)
        c = this.row(2)
        d = this.row(3)
        inv = 1.0 / det
        # Row `i` of the adjugate is column `i` of the inverse's transpose, so
        # each column of the inverse below lists row `i`'s cofactors.
        return Some(Mat4(
            Vec4(
                b.y * m.c5 - b.z * m.c4 + b.w * m.c3,
                -b.x * m.c5 + b.z * m.c2 - b.w * m.c1,
                b.x * m.c4 - b.y * m.c2 + b.w * m.c0,
                -b.x * m.c3 + b.y * m.c1 - b.z * m.c0,
            ) * inv,
            Vec4(
                -a.y * m.c5 + a.z * m.c4 - a.w * m.c3,
                a.x * m.c5 - a.z * m.c2 + a.w * m.c1,
                -a.x * m.c4 + a.y * m.c2 - a.w * m.c0,
                a.x * m.c3 - a.y * m.c1 + a.z * m.c0,
            ) * inv,
            Vec4(
                d.y * m.s5 - d.z * m.s4 + d.w * m.s3,
                -d.x * m.s5 + d.z * m.s2 - d.w * m.s1,
                d.x * m.s4 - d.y * m.s2 + d.w * m.s0,
                -d.x * m.s3 + d.y * m.s1 - d.z * m.s0,
            ) * inv,
            Vec4(
                -c.y * m.s5 + c.z * m.s4 - c.w * m.s3,
                c.x * m.s5 - c.z * m.s2 + c.w * m.s1,
                -c.x * m.s4 + c.y * m.s2 - c.w * m.s0,
                c.x * m.s3 - c.y * m.s1 + c.z * m.s0,
            ) * inv,
        ))

    ## The inverse; a matrix with none panics.
    pub fn inverse() -> Mat4:
        match this.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

    ## The point `p` transformed: `p` with `w` 1, divided by the result's
    ## `w` (1 for any matrix that is not a projection).
    pub fn transform_point3(p: Vec3) -> Vec3:
        r = this * p.extend(1.0)
        return r.truncate() / r.w

    ## The direction `v` transformed: `v` with `w` 0, so no translation.
    pub fn transform_vector3(v: Vec3) -> Vec3:
        return (this * v.extend(0.0)).truncate()

## The 2×2 minors of a `Mat4`'s top two rows (`s0`…`s5`) and bottom two
## (`c0`…`c5`), from which its determinant and inverse are built.
@derive(Copy)
struct MinorPairs:
    s0: f32
    s1: f32
    s2: f32
    s3: f32
    s4: f32
    s5: f32
    c0: f32
    c1: f32
    c2: f32
    c3: f32
    c4: f32
    c5: f32

    static fn of(m: Mat4) -> MinorPairs:
        a = m.row(0)
        b = m.row(1)
        c = m.row(2)
        d = m.row(3)
        return MinorPairs(
            a.x * b.y - b.x * a.y,
            a.x * b.z - b.x * a.z,
            a.x * b.w - b.x * a.w,
            a.y * b.z - b.y * a.z,
            a.y * b.w - b.y * a.w,
            a.z * b.w - b.z * a.w,
            c.x * d.y - d.x * c.y,
            c.x * d.z - d.x * c.z,
            c.x * d.w - d.x * c.w,
            c.y * d.z - d.y * c.z,
            c.y * d.w - d.y * c.w,
            c.z * d.w - d.z * c.w,
        )

    fn determinant() -> f32:
        return this.s0 * this.c5 - this.s1 * this.c4 + this.s2 * this.c3 + this.s3 * this.c2 - this.s4 * this.c1 + this.s5 * this.c0

extend Mat4 implements Add, Sub, Mul, Neg:
    type Output = Mat4

    fn add(o: Mat4) -> Mat4:
        return Mat4(this.x_axis + o.x_axis, this.y_axis + o.y_axis, this.z_axis + o.z_axis, this.w_axis + o.w_axis)

    fn sub(o: Mat4) -> Mat4:
        return Mat4(this.x_axis - o.x_axis, this.y_axis - o.y_axis, this.z_axis - o.z_axis, this.w_axis - o.w_axis)

    fn mul(o: Mat4) -> Mat4:
        return Mat4(this * o.x_axis, this * o.y_axis, this * o.z_axis, this * o.w_axis)

    fn neg() -> Mat4:
        return Mat4(-this.x_axis, -this.y_axis, -this.z_axis, -this.w_axis)

extend Mat4 implements Mul[Vec4]:
    type Output = Vec4

    fn mul(v: Vec4) -> Vec4:
        r = this.y_axis.mul_add(Vec4.splat(v.y), this.x_axis * v.x)
        r = this.z_axis.mul_add(Vec4.splat(v.z), r)
        return this.w_axis.mul_add(Vec4.splat(v.w), r)

extend Mat4 implements Mul[f32]:
    type Output = Mat4

    fn mul(s: f32) -> Mat4:
        return Mat4(this.x_axis * s, this.y_axis * s, this.z_axis * s, this.w_axis * s)

extend f32 implements Mul[Mat4]:
    type Output = Mat4

    fn mul(m: Mat4) -> Mat4:
        return m * this

## -- rotations and transforms (`[STD-28]`, ODR-043) ---------------------------

## A rotation in space, as a quaternion `x i + y j + z k + w` of length one.
## `q * p` rotates by `p`, then by `q`; `q * v` rotates the vector `v`.
@derive(Copy)
@layout(c)
pub struct Quat:
    pub x: f32
    pub y: f32
    pub z: f32
    pub w: f32

    pub const IDENTITY: Quat = Quat(0.0, 0.0, 0.0, 1.0)

    ## A rotation by `angle` radians about `axis`, which has length one.
    pub static fn from_axis_angle(axis: Vec3, angle: f32) -> Quat:
        half = angle * 0.5
        s = half.sin()
        return Quat(axis.x * s, axis.y * s, axis.z * s, half.cos())

    pub static fn from_rotation_x(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(1.0, 0.0, 0.0), angle)

    pub static fn from_rotation_y(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(0.0, 1.0, 0.0), angle)

    pub static fn from_rotation_z(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(0.0, 0.0, 1.0), angle)

    pub fn dot(o: Quat) -> f32:
        return this.x * o.x + this.y * o.y + this.z * o.z + this.w * o.w

    pub fn length() -> f32:
        return this.dot(this).sqrt()

    ## The same rotation, of length one; a zero quaternion panics.
    pub fn normalize() -> Quat:
        length = this.length()
        if length == 0.0:
            panic("normalize of a zero quaternion")
        inv = 1.0 / length
        return Quat(this.x * inv, this.y * inv, this.z * inv, this.w * inv)

    ## The opposite rotation, for a quaternion of length one.
    pub fn conjugate() -> Quat:
        return Quat(-this.x, -this.y, -this.z, this.w)

    ## The opposite rotation, for a quaternion of any length but zero.
    pub fn inverse() -> Quat:
        inv = 1.0 / this.dot(this)
        return Quat(-this.x * inv, -this.y * inv, -this.z * inv, this.w * inv)

    ## The rotation a fraction `t` of the way from this one to `end`, along
    ## the shorter arc at a steady speed.
    pub fn slerp(end: Quat, t: f32) -> Quat:
        cos_theta = this.dot(end)
        e = end
        if cos_theta < 0.0:
            e = Quat(-end.x, -end.y, -end.z, -end.w)
            cos_theta = -cos_theta
        # Nearly one rotation: the arc is too short to measure, so a straight
        # line serves.
        if cos_theta > 0.9995:
            return Quat(
                this.x + (e.x - this.x) * t,
                this.y + (e.y - this.y) * t,
                this.z + (e.z - this.z) * t,
                this.w + (e.w - this.w) * t,
            ).normalize()
        theta = cos_theta.acos()
        sin_theta = theta.sin()
        a = ((1.0 - t) * theta).sin() / sin_theta
        b = (t * theta).sin() / sin_theta
        return Quat(this.x * a + e.x * b, this.y * a + e.y * b, this.z * a + e.z * b, this.w * a + e.w * b)

extend Quat implements Mul:
    type Output = Quat

    fn mul(o: Quat) -> Quat:
        return Quat(
            this.w * o.x + this.x * o.w + this.y * o.z - this.z * o.y,
            this.w * o.y - this.x * o.z + this.y * o.w + this.z * o.x,
            this.w * o.z + this.x * o.y - this.y * o.x + this.z * o.w,
            this.w * o.w - this.x * o.x - this.y * o.y - this.z * o.z,
        )

extend Quat implements Mul[Vec3]:
    type Output = Vec3

    ## `v` rotated: `v + w t + q × t` with `t = 2 q × v`, `q` the vector part,
    ## `w t` added with one rounding (`[STD-3]`).
    fn mul(v: Vec3) -> Vec3:
        q = Vec3(this.x, this.y, this.z)
        t = q.cross(v) * 2.0
        return t.mul_add(Vec3.splat(this.w), v + q.cross(t))

## A scale, then a rotation, then a move.
@derive(Copy)
@layout(c)
pub struct Transform:
    pub translation: Vec3
    pub rotation: Quat
    pub scale: Vec3

    pub const IDENTITY: Transform = Transform(Vec3(0.0, 0.0, 0.0), Quat(0.0, 0.0, 0.0, 1.0), Vec3(1.0, 1.0, 1.0))

    pub static fn from_translation(t: Vec3) -> Transform:
        return Transform(t, Quat(0.0, 0.0, 0.0, 1.0), Vec3(1.0, 1.0, 1.0))

    pub static fn from_rotation(q: Quat) -> Transform:
        return Transform(Vec3(0.0, 0.0, 0.0), q, Vec3(1.0, 1.0, 1.0))

    pub static fn from_scale(s: Vec3) -> Transform:
        return Transform(Vec3(0.0, 0.0, 0.0), Quat(0.0, 0.0, 0.0, 1.0), s)

    ## The point `p`, scaled, rotated and moved.
    pub fn transform_point(p: Vec3) -> Vec3:
        return this.rotation * (this.scale * p) + this.translation

    ## The direction `v`, scaled and rotated but not moved.
    pub fn transform_vector(v: Vec3) -> Vec3:
        return this.rotation * (this.scale * v)

    pub fn to_mat4() -> Mat4:
        return Mat4.from_scale_rotation_translation(this.scale, this.rotation, this.translation)

extend Transform implements Mul:
    type Output = Transform

    ## `o`, then this one. With a scale that is not the same along every
    ## axis and a rotation, the result is the usual approximation: scales
    ## multiply axis by axis.
    fn mul(o: Transform) -> Transform:
        return Transform(this.transform_point(o.translation), this.rotation * o.rotation, this.scale * o.scale)

## -- shapes (`[STD-28]`, ODR-043) ----------------------------------------------

## An axis-aligned box: the points between `min` and `max` in every axis.
@derive(Copy)
@layout(c)
pub struct Aabb:
    pub min: Vec3
    pub max: Vec3

    pub static fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Aabb:
        return Aabb(center - half_extents, center + half_extents)

    pub fn center() -> Vec3:
        return (this.min + this.max) * 0.5

    pub fn half_extents() -> Vec3:
        return (this.max - this.min) * 0.5

    ## Whether `p` is inside, or on the boundary.
    pub fn contains(p: Vec3) -> bool:
        return this.min.x <= p.x and p.x <= this.max.x and this.min.y <= p.y and p.y <= this.max.y and this.min.z <= p.z and p.z <= this.max.z

    ## Whether the two boxes share a point.
    pub fn intersects(o: Aabb) -> bool:
        return this.min.x <= o.max.x and o.min.x <= this.max.x and this.min.y <= o.max.y and o.min.y <= this.max.y and this.min.z <= o.max.z and o.min.z <= this.max.z

    ## The least box holding both.
    pub fn union(o: Aabb) -> Aabb:
        return Aabb(this.min.min(o.min), this.max.max(o.max))

    ## The least box holding this one and `p`.
    pub fn expand(p: Vec3) -> Aabb:
        return Aabb(this.min.min(p), this.max.max(p))

## A ball: the points no further than `radius` from `center`.
@derive(Copy)
@layout(c)
pub struct Sphere:
    pub center: Vec3
    pub radius: f32

    pub fn contains(p: Vec3) -> bool:
        return (p - this.center).length_squared() <= this.radius * this.radius

    pub fn intersects(o: Sphere) -> bool:
        reach = this.radius + o.radius
        return (o.center - this.center).length_squared() <= reach * reach

    ## Whether the ball and the box share a point: the box's point nearest
    ## the center is within `radius`.
    pub fn intersects_aabb(b: Aabb) -> bool:
        nearest = this.center.max(b.min).min(b.max)
        return (nearest - this.center).length_squared() <= this.radius * this.radius

## A half-line: `origin + direction * t` for every `t ≥ 0`.
@derive(Copy)
@layout(c)
pub struct Ray:
    pub origin: Vec3
    pub direction: Vec3

    ## The point at `t`.
    pub fn at(t: f32) -> Vec3:
        return this.origin + this.direction * t

    ## The least `t ≥ 0` at which the ray is on the plane, or `None`: 0 when
    ## it starts on it.
    pub fn intersect_plane(p: Plane) -> Option[f32]:
        distance = p.signed_distance(this.origin)
        if distance == 0.0:
            return Some(0.0)
        facing = p.normal.dot(this.direction)
        if facing == 0.0:
            return None
        t = -distance / facing
        if t >= 0.0:
            return Some(t)
        return None

    ## The least `t ≥ 0` at which the ray is in the ball, or `None`: 0 when
    ## it starts inside.
    pub fn intersect_sphere(s: Sphere) -> Option[f32]:
        to_origin = this.origin - s.center
        c = to_origin.length_squared() - s.radius * s.radius
        if c <= 0.0:
            return Some(0.0)
        a = this.direction.length_squared()
        if a == 0.0:
            return None
        b = to_origin.dot(this.direction)
        discriminant = b * b - a * c
        if discriminant < 0.0:
            return None
        # From outside, the nearer root is where the ray enters; a negative
        # one means the ball is behind it.
        near = (-b - discriminant.sqrt()) / a
        if near >= 0.0:
            return Some(near)
        return None

    ## The least `t ≥ 0` at which the ray is in the box, or `None`: the
    ## overlap of the three slabs' intervals, 0 when it starts inside.
    pub fn intersect_aabb(b: Aabb) -> Option[f32]:
        low: f32 = 0.0
        high = f32.INF
        (l, h) = slab(this.origin.x, this.direction.x, b.min.x, b.max.x)
        low = max(low, l)
        high = min(high, h)
        (l, h) = slab(this.origin.y, this.direction.y, b.min.y, b.max.y)
        low = max(low, l)
        high = min(high, h)
        (l, h) = slab(this.origin.z, this.direction.z, b.min.z, b.max.z)
        low = max(low, l)
        high = min(high, h)
        if low <= high:
            return Some(low)
        return None

## Where a ray crosses one axis's slab `[lo, hi]`, as the interval of `t`;
## parallel to it, everything or nothing.
fn slab(origin: f32, direction: f32, lo: f32, hi: f32) -> (f32, f32):
    if direction == 0.0:
        if lo <= origin and origin <= hi:
            return (-f32.INF, f32.INF)
        return (f32.INF, -f32.INF)
    t1 = (lo - origin) / direction
    t2 = (hi - origin) / direction
    return (min(t1, t2), max(t1, t2))

## The points `p` with `normal.dot(p) + d == 0`; `normal.dot(p) + d` is `p`'s
## distance from it, above it on the side `normal` points to, for a `normal`
## of length one.
@derive(Copy)
@layout(c)
pub struct Plane:
    pub normal: Vec3
    pub d: f32

    ## The plane through `p` facing `normal`, which is made of length one.
    pub static fn from_point_normal(p: Vec3, normal: Vec3) -> Plane:
        n = normal.normalize()
        return Plane(n, -n.dot(p))

    ## The plane through three points, facing the side from which they run
    ## counterclockwise.
    pub static fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Plane:
        n = (b - a).cross(c - a).normalize()
        return Plane(n, -n.dot(a))

    pub fn signed_distance(p: Vec3) -> f32:
        return this.normal.dot(p) + this.d

    ## The same plane, with a normal of length one.
    pub fn normalize() -> Plane:
        length = this.normal.length()
        if length == 0.0:
            panic("normalize of a plane with no normal")
        return Plane(this.normal / length, this.d / length)

## The six planes around what a camera sees, each facing in.
@derive(Copy)
@layout(c)
pub struct Frustum:
    pub left: Plane
    pub right: Plane
    pub bottom: Plane
    pub top: Plane
    pub near: Plane
    pub far: Plane

    ## The planes of `m`, a `perspective_rh` or `orthographic_rh` projection
    ## times a view: a point is inside when `m * p` lies in clip space's box,
    ## `-w ≤ x ≤ w`, `-w ≤ y ≤ w`, `0 ≤ z ≤ w`.
    pub static fn from_view_projection(m: Mat4) -> Frustum:
        r0 = m.row(0)
        r1 = m.row(1)
        r2 = m.row(2)
        r3 = m.row(3)
        return Frustum(
            clip_plane(r3 + r0),
            clip_plane(r3 - r0),
            clip_plane(r3 + r1),
            clip_plane(r3 - r1),
            clip_plane(r2),
            clip_plane(r3 - r2),
        )

    pub fn contains_point(p: Vec3) -> bool:
        return this.left.signed_distance(p) >= 0.0 and this.right.signed_distance(p) >= 0.0 and this.bottom.signed_distance(p) >= 0.0 and this.top.signed_distance(p) >= 0.0 and this.near.signed_distance(p) >= 0.0 and this.far.signed_distance(p) >= 0.0

    ## Whether the ball may be seen: no plane has it wholly outside.
    pub fn intersects_sphere(s: Sphere) -> bool:
        r = -s.radius
        return this.left.signed_distance(s.center) >= r and this.right.signed_distance(s.center) >= r and this.bottom.signed_distance(s.center) >= r and this.top.signed_distance(s.center) >= r and this.near.signed_distance(s.center) >= r and this.far.signed_distance(s.center) >= r

    ## Whether the box may be seen: no plane has its corner furthest along
    ## the plane's normal outside.
    pub fn intersects_aabb(b: Aabb) -> bool:
        return reaches(this.left, b) and reaches(this.right, b) and reaches(this.bottom, b) and reaches(this.top, b) and reaches(this.near, b) and reaches(this.far, b)

## A clip-space row `(a, b, c, d)` as a plane of length-one normal.
fn clip_plane(row: Vec4) -> Plane:
    return Plane(row.truncate(), row.w).normalize()

## Whether the box's corner furthest along the plane's normal is inside.
fn reaches(p: Plane, b: Aabb) -> bool:
    x = b.max.x if p.normal.x >= 0.0 else b.min.x
    y = b.max.y if p.normal.y >= 0.0 else b.min.y
    z = b.max.z if p.normal.z >= 0.0 else b.min.z
    return p.signed_distance(Vec3(x, y, z)) >= 0.0

## -- compensated summation (`[STD-5]`, ODR-043) -----------------------------------

## A running sum of `f64`s that keeps the low-order bits each addition drops,
## and adds them back at the end (Neumaier's form of Kahan's method, which
## also holds when an addend is larger than the sum so far).
pub struct KahanSum:
    sum: f64 = 0.0
    lost: f64 = 0.0

    ## An empty sum, zero.
    pub static fn new() -> KahanSum:
        return KahanSum()

    pub fn add(x: f64):
        t = this.sum + x
        if abs(this.sum) >= abs(x):
            this.lost += (this.sum - t) + x
        else:
            this.lost += (x - t) + this.sum
        this.sum = t

    pub fn value() -> f64:
        return this.sum + this.lost

    ## The sum of `xs`, compensated.
    pub static fn of(xs: Span[f64]) -> f64:
        k = KahanSum.new()
        for x in xs:
            k.add(x)
        return k.value()
