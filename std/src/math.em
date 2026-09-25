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
## turn it into one.
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

extend i128 implements Number:
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

extend u128 implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend usize implements Number:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

## `f16` is not a `Float` (C has no maths functions for it); its answers come
## back as `f32`, which holds every `f16` exactly (ODR-041).
extend f16 implements Number:
    type Real = f32
    fn to_real(self) -> f32:
        return self as f32

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

## `a * b + c` rounded once, not twice: a fused multiply-add, the same on
## every target (`[STD-3]`). It is `a.mul_add(b, c)`, C's `fma`.
pub fn fma[T: Number](a: T, b: T, c: T) -> T.Real:
    return a.to_real().mul_add(b.to_real(), c.to_real())

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
    pub fn splat(v: f32) -> Vec2:
        return Vec2(v, v)

    pub fn dot(self, o: Vec2) -> f32:
        return self.x * o.x + self.y * o.y

    ## The smaller of each pair of components.
    pub fn min(self, o: Vec2) -> Vec2:
        return Vec2(min(self.x, o.x), min(self.y, o.y))

    ## The larger of each pair of components.
    pub fn max(self, o: Vec2) -> Vec2:
        return Vec2(max(self.x, o.x), max(self.y, o.y))

    pub fn abs(self) -> Vec2:
        return Vec2(abs(self.x), abs(self.y))

    pub fn length_squared(self) -> f32:
        return self.dot(self)

    pub fn length(self) -> f32:
        return self.length_squared().sqrt()

    pub fn distance(self, o: Vec2) -> f32:
        return (self - o).length()

    pub fn distance_squared(self, o: Vec2) -> f32:
        return (self - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize(self) -> Vec2:
        length = self.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return self / length

    pub fn normalize_or_zero(self) -> Vec2:
        length = self.length()
        if length == 0.0:
            return Vec2.ZERO
        return self / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(self, a: Vec2, b: Vec2) -> Vec2:
        return Vec2(self.x.mul_add(a.x, b.x), self.y.mul_add(a.y, b.y))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(self, o: Vec2, t: f32) -> Vec2:
        return self + (o - self) * t

    ## This vector with a z component.
    pub fn extend(self, z: f32) -> Vec3:
        return Vec3(self.x, self.y, z)

extend Vec2 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec2

    fn add(self, o: Vec2) -> Vec2:
        return Vec2(self.x + o.x, self.y + o.y)

    fn sub(self, o: Vec2) -> Vec2:
        return Vec2(self.x - o.x, self.y - o.y)

    fn mul(self, o: Vec2) -> Vec2:
        return Vec2(self.x * o.x, self.y * o.y)

    fn div(self, o: Vec2) -> Vec2:
        return Vec2(self.x / o.x, self.y / o.y)

    fn neg(self) -> Vec2:
        return Vec2(-self.x, -self.y)

extend Vec2 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec2

    fn add(self, s: f32) -> Vec2:
        return Vec2(self.x + s, self.y + s)

    fn sub(self, s: f32) -> Vec2:
        return Vec2(self.x - s, self.y - s)

    fn mul(self, s: f32) -> Vec2:
        return Vec2(self.x * s, self.y * s)

    fn div(self, s: f32) -> Vec2:
        return Vec2(self.x / s, self.y / s)

extend f32 implements Add[Vec2], Sub[Vec2], Mul[Vec2], Div[Vec2]:
    type Output = Vec2

    fn add(self, v: Vec2) -> Vec2:
        return Vec2(self + v.x, self + v.y)

    fn sub(self, v: Vec2) -> Vec2:
        return Vec2(self - v.x, self - v.y)

    fn mul(self, v: Vec2) -> Vec2:
        return Vec2(self * v.x, self * v.y)

    fn div(self, v: Vec2) -> Vec2:
        return Vec2(self / v.x, self / v.y)

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
    pub fn splat(v: f32) -> Vec3:
        return Vec3(v, v, v)

    pub fn dot(self, o: Vec3) -> f32:
        return self.x * o.x + self.y * o.y + self.z * o.z

    ## The smaller of each pair of components.
    pub fn min(self, o: Vec3) -> Vec3:
        return Vec3(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z))

    ## The larger of each pair of components.
    pub fn max(self, o: Vec3) -> Vec3:
        return Vec3(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z))

    pub fn abs(self) -> Vec3:
        return Vec3(abs(self.x), abs(self.y), abs(self.z))

    pub fn length_squared(self) -> f32:
        return self.dot(self)

    pub fn length(self) -> f32:
        return self.length_squared().sqrt()

    pub fn distance(self, o: Vec3) -> f32:
        return (self - o).length()

    pub fn distance_squared(self, o: Vec3) -> f32:
        return (self - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize(self) -> Vec3:
        length = self.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return self / length

    pub fn normalize_or_zero(self) -> Vec3:
        length = self.length()
        if length == 0.0:
            return Vec3.ZERO
        return self / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(self, a: Vec3, b: Vec3) -> Vec3:
        return Vec3(self.x.mul_add(a.x, b.x), self.y.mul_add(a.y, b.y), self.z.mul_add(a.z, b.z))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(self, o: Vec3, t: f32) -> Vec3:
        return self + (o - self) * t

    ## The vector at right angles to both, by the right-hand rule.
    pub fn cross(self, o: Vec3) -> Vec3:
        return Vec3(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)

    ## This vector with a w component.
    pub fn extend(self, w: f32) -> Vec4:
        return Vec4(self.x, self.y, self.z, w)

    ## This vector without its last component.
    pub fn truncate(self) -> Vec2:
        return Vec2(self.x, self.y)

extend Vec3 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec3

    fn add(self, o: Vec3) -> Vec3:
        return Vec3(self.x + o.x, self.y + o.y, self.z + o.z)

    fn sub(self, o: Vec3) -> Vec3:
        return Vec3(self.x - o.x, self.y - o.y, self.z - o.z)

    fn mul(self, o: Vec3) -> Vec3:
        return Vec3(self.x * o.x, self.y * o.y, self.z * o.z)

    fn div(self, o: Vec3) -> Vec3:
        return Vec3(self.x / o.x, self.y / o.y, self.z / o.z)

    fn neg(self) -> Vec3:
        return Vec3(-self.x, -self.y, -self.z)

extend Vec3 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec3

    fn add(self, s: f32) -> Vec3:
        return Vec3(self.x + s, self.y + s, self.z + s)

    fn sub(self, s: f32) -> Vec3:
        return Vec3(self.x - s, self.y - s, self.z - s)

    fn mul(self, s: f32) -> Vec3:
        return Vec3(self.x * s, self.y * s, self.z * s)

    fn div(self, s: f32) -> Vec3:
        return Vec3(self.x / s, self.y / s, self.z / s)

extend f32 implements Add[Vec3], Sub[Vec3], Mul[Vec3], Div[Vec3]:
    type Output = Vec3

    fn add(self, v: Vec3) -> Vec3:
        return Vec3(self + v.x, self + v.y, self + v.z)

    fn sub(self, v: Vec3) -> Vec3:
        return Vec3(self - v.x, self - v.y, self - v.z)

    fn mul(self, v: Vec3) -> Vec3:
        return Vec3(self * v.x, self * v.y, self * v.z)

    fn div(self, v: Vec3) -> Vec3:
        return Vec3(self / v.x, self / v.y, self / v.z)

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
    pub fn splat(v: f32) -> Vec4:
        return Vec4(v, v, v, v)

    pub fn dot(self, o: Vec4) -> f32:
        return self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w

    ## The smaller of each pair of components.
    pub fn min(self, o: Vec4) -> Vec4:
        return Vec4(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z), min(self.w, o.w))

    ## The larger of each pair of components.
    pub fn max(self, o: Vec4) -> Vec4:
        return Vec4(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z), max(self.w, o.w))

    pub fn abs(self) -> Vec4:
        return Vec4(abs(self.x), abs(self.y), abs(self.z), abs(self.w))

    pub fn length_squared(self) -> f32:
        return self.dot(self)

    pub fn length(self) -> f32:
        return self.length_squared().sqrt()

    pub fn distance(self, o: Vec4) -> f32:
        return (self - o).length()

    pub fn distance_squared(self, o: Vec4) -> f32:
        return (self - o).length_squared()

    ## The same direction, of length one. A zero vector has no direction,
    ## and panics; `normalize_or_zero` gives it back.
    pub fn normalize(self) -> Vec4:
        length = self.length()
        if length == 0.0:
            panic("normalize of a zero vector; use normalize_or_zero")
        return self / length

    pub fn normalize_or_zero(self) -> Vec4:
        length = self.length()
        if length == 0.0:
            return Vec4.ZERO
        return self / length

    ## `self * a + b` component by component, each rounded once, not
    ## twice (`[STD-3]`).
    pub fn mul_add(self, a: Vec4, b: Vec4) -> Vec4:
        return Vec4(self.x.mul_add(a.x, b.x), self.y.mul_add(a.y, b.y), self.z.mul_add(a.z, b.z), self.w.mul_add(a.w, b.w))

    ## The point a fraction `t` of the way from this one to `o`.
    pub fn lerp(self, o: Vec4, t: f32) -> Vec4:
        return self + (o - self) * t

    ## This vector without its last component.
    pub fn truncate(self) -> Vec3:
        return Vec3(self.x, self.y, self.z)

extend Vec4 implements Add, Sub, Mul, Div, Neg:
    type Output = Vec4

    fn add(self, o: Vec4) -> Vec4:
        return Vec4(self.x + o.x, self.y + o.y, self.z + o.z, self.w + o.w)

    fn sub(self, o: Vec4) -> Vec4:
        return Vec4(self.x - o.x, self.y - o.y, self.z - o.z, self.w - o.w)

    fn mul(self, o: Vec4) -> Vec4:
        return Vec4(self.x * o.x, self.y * o.y, self.z * o.z, self.w * o.w)

    fn div(self, o: Vec4) -> Vec4:
        return Vec4(self.x / o.x, self.y / o.y, self.z / o.z, self.w / o.w)

    fn neg(self) -> Vec4:
        return Vec4(-self.x, -self.y, -self.z, -self.w)

extend Vec4 implements Add[f32], Sub[f32], Mul[f32], Div[f32]:
    type Output = Vec4

    fn add(self, s: f32) -> Vec4:
        return Vec4(self.x + s, self.y + s, self.z + s, self.w + s)

    fn sub(self, s: f32) -> Vec4:
        return Vec4(self.x - s, self.y - s, self.z - s, self.w - s)

    fn mul(self, s: f32) -> Vec4:
        return Vec4(self.x * s, self.y * s, self.z * s, self.w * s)

    fn div(self, s: f32) -> Vec4:
        return Vec4(self.x / s, self.y / s, self.z / s, self.w / s)

extend f32 implements Add[Vec4], Sub[Vec4], Mul[Vec4], Div[Vec4]:
    type Output = Vec4

    fn add(self, v: Vec4) -> Vec4:
        return Vec4(self + v.x, self + v.y, self + v.z, self + v.w)

    fn sub(self, v: Vec4) -> Vec4:
        return Vec4(self - v.x, self - v.y, self - v.z, self - v.w)

    fn mul(self, v: Vec4) -> Vec4:
        return Vec4(self * v.x, self * v.y, self * v.z, self * v.w)

    fn div(self, v: Vec4) -> Vec4:
        return Vec4(self / v.x, self / v.y, self / v.z, self / v.w)

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
    pub fn splat(v: i32) -> IVec2:
        return IVec2(v, v)

    pub fn dot(self, o: IVec2) -> i32:
        return self.x * o.x + self.y * o.y

    ## The smaller of each pair of components.
    pub fn min(self, o: IVec2) -> IVec2:
        return IVec2(min(self.x, o.x), min(self.y, o.y))

    ## The larger of each pair of components.
    pub fn max(self, o: IVec2) -> IVec2:
        return IVec2(max(self.x, o.x), max(self.y, o.y))

    pub fn abs(self) -> IVec2:
        return IVec2(abs(self.x), abs(self.y))

    ## This vector with a z component.
    pub fn extend(self, z: i32) -> IVec3:
        return IVec3(self.x, self.y, z)

extend IVec2 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec2

    fn add(self, o: IVec2) -> IVec2:
        return IVec2(self.x + o.x, self.y + o.y)

    fn sub(self, o: IVec2) -> IVec2:
        return IVec2(self.x - o.x, self.y - o.y)

    fn mul(self, o: IVec2) -> IVec2:
        return IVec2(self.x * o.x, self.y * o.y)

    fn floordiv(self, o: IVec2) -> IVec2:
        return IVec2(self.x // o.x, self.y // o.y)

    fn rem(self, o: IVec2) -> IVec2:
        return IVec2(self.x % o.x, self.y % o.y)

    fn neg(self) -> IVec2:
        return IVec2(-self.x, -self.y)

extend IVec2 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec2

    fn add(self, s: i32) -> IVec2:
        return IVec2(self.x + s, self.y + s)

    fn sub(self, s: i32) -> IVec2:
        return IVec2(self.x - s, self.y - s)

    fn mul(self, s: i32) -> IVec2:
        return IVec2(self.x * s, self.y * s)

    fn floordiv(self, s: i32) -> IVec2:
        return IVec2(self.x // s, self.y // s)

    fn rem(self, s: i32) -> IVec2:
        return IVec2(self.x % s, self.y % s)

extend i32 implements Add[IVec2], Sub[IVec2], Mul[IVec2], FloorDiv[IVec2], Rem[IVec2]:
    type Output = IVec2

    fn add(self, v: IVec2) -> IVec2:
        return IVec2(self + v.x, self + v.y)

    fn sub(self, v: IVec2) -> IVec2:
        return IVec2(self - v.x, self - v.y)

    fn mul(self, v: IVec2) -> IVec2:
        return IVec2(self * v.x, self * v.y)

    fn floordiv(self, v: IVec2) -> IVec2:
        return IVec2(self // v.x, self // v.y)

    fn rem(self, v: IVec2) -> IVec2:
        return IVec2(self % v.x, self % v.y)

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
    pub fn splat(v: i32) -> IVec3:
        return IVec3(v, v, v)

    pub fn dot(self, o: IVec3) -> i32:
        return self.x * o.x + self.y * o.y + self.z * o.z

    ## The smaller of each pair of components.
    pub fn min(self, o: IVec3) -> IVec3:
        return IVec3(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z))

    ## The larger of each pair of components.
    pub fn max(self, o: IVec3) -> IVec3:
        return IVec3(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z))

    pub fn abs(self) -> IVec3:
        return IVec3(abs(self.x), abs(self.y), abs(self.z))

    ## This vector with a w component.
    pub fn extend(self, w: i32) -> IVec4:
        return IVec4(self.x, self.y, self.z, w)

    ## This vector without its last component.
    pub fn truncate(self) -> IVec2:
        return IVec2(self.x, self.y)

extend IVec3 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec3

    fn add(self, o: IVec3) -> IVec3:
        return IVec3(self.x + o.x, self.y + o.y, self.z + o.z)

    fn sub(self, o: IVec3) -> IVec3:
        return IVec3(self.x - o.x, self.y - o.y, self.z - o.z)

    fn mul(self, o: IVec3) -> IVec3:
        return IVec3(self.x * o.x, self.y * o.y, self.z * o.z)

    fn floordiv(self, o: IVec3) -> IVec3:
        return IVec3(self.x // o.x, self.y // o.y, self.z // o.z)

    fn rem(self, o: IVec3) -> IVec3:
        return IVec3(self.x % o.x, self.y % o.y, self.z % o.z)

    fn neg(self) -> IVec3:
        return IVec3(-self.x, -self.y, -self.z)

extend IVec3 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec3

    fn add(self, s: i32) -> IVec3:
        return IVec3(self.x + s, self.y + s, self.z + s)

    fn sub(self, s: i32) -> IVec3:
        return IVec3(self.x - s, self.y - s, self.z - s)

    fn mul(self, s: i32) -> IVec3:
        return IVec3(self.x * s, self.y * s, self.z * s)

    fn floordiv(self, s: i32) -> IVec3:
        return IVec3(self.x // s, self.y // s, self.z // s)

    fn rem(self, s: i32) -> IVec3:
        return IVec3(self.x % s, self.y % s, self.z % s)

extend i32 implements Add[IVec3], Sub[IVec3], Mul[IVec3], FloorDiv[IVec3], Rem[IVec3]:
    type Output = IVec3

    fn add(self, v: IVec3) -> IVec3:
        return IVec3(self + v.x, self + v.y, self + v.z)

    fn sub(self, v: IVec3) -> IVec3:
        return IVec3(self - v.x, self - v.y, self - v.z)

    fn mul(self, v: IVec3) -> IVec3:
        return IVec3(self * v.x, self * v.y, self * v.z)

    fn floordiv(self, v: IVec3) -> IVec3:
        return IVec3(self // v.x, self // v.y, self // v.z)

    fn rem(self, v: IVec3) -> IVec3:
        return IVec3(self % v.x, self % v.y, self % v.z)

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
    pub fn splat(v: i32) -> IVec4:
        return IVec4(v, v, v, v)

    pub fn dot(self, o: IVec4) -> i32:
        return self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w

    ## The smaller of each pair of components.
    pub fn min(self, o: IVec4) -> IVec4:
        return IVec4(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z), min(self.w, o.w))

    ## The larger of each pair of components.
    pub fn max(self, o: IVec4) -> IVec4:
        return IVec4(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z), max(self.w, o.w))

    pub fn abs(self) -> IVec4:
        return IVec4(abs(self.x), abs(self.y), abs(self.z), abs(self.w))

    ## This vector without its last component.
    pub fn truncate(self) -> IVec3:
        return IVec3(self.x, self.y, self.z)

extend IVec4 implements Add, Sub, Mul, FloorDiv, Rem, Neg:
    type Output = IVec4

    fn add(self, o: IVec4) -> IVec4:
        return IVec4(self.x + o.x, self.y + o.y, self.z + o.z, self.w + o.w)

    fn sub(self, o: IVec4) -> IVec4:
        return IVec4(self.x - o.x, self.y - o.y, self.z - o.z, self.w - o.w)

    fn mul(self, o: IVec4) -> IVec4:
        return IVec4(self.x * o.x, self.y * o.y, self.z * o.z, self.w * o.w)

    fn floordiv(self, o: IVec4) -> IVec4:
        return IVec4(self.x // o.x, self.y // o.y, self.z // o.z, self.w // o.w)

    fn rem(self, o: IVec4) -> IVec4:
        return IVec4(self.x % o.x, self.y % o.y, self.z % o.z, self.w % o.w)

    fn neg(self) -> IVec4:
        return IVec4(-self.x, -self.y, -self.z, -self.w)

extend IVec4 implements Add[i32], Sub[i32], Mul[i32], FloorDiv[i32], Rem[i32]:
    type Output = IVec4

    fn add(self, s: i32) -> IVec4:
        return IVec4(self.x + s, self.y + s, self.z + s, self.w + s)

    fn sub(self, s: i32) -> IVec4:
        return IVec4(self.x - s, self.y - s, self.z - s, self.w - s)

    fn mul(self, s: i32) -> IVec4:
        return IVec4(self.x * s, self.y * s, self.z * s, self.w * s)

    fn floordiv(self, s: i32) -> IVec4:
        return IVec4(self.x // s, self.y // s, self.z // s, self.w // s)

    fn rem(self, s: i32) -> IVec4:
        return IVec4(self.x % s, self.y % s, self.z % s, self.w % s)

extend i32 implements Add[IVec4], Sub[IVec4], Mul[IVec4], FloorDiv[IVec4], Rem[IVec4]:
    type Output = IVec4

    fn add(self, v: IVec4) -> IVec4:
        return IVec4(self + v.x, self + v.y, self + v.z, self + v.w)

    fn sub(self, v: IVec4) -> IVec4:
        return IVec4(self - v.x, self - v.y, self - v.z, self - v.w)

    fn mul(self, v: IVec4) -> IVec4:
        return IVec4(self * v.x, self * v.y, self * v.z, self * v.w)

    fn floordiv(self, v: IVec4) -> IVec4:
        return IVec4(self // v.x, self // v.y, self // v.z, self // v.w)

    fn rem(self, v: IVec4) -> IVec4:
        return IVec4(self % v.x, self % v.y, self % v.z, self % v.w)

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
    pub fn splat(v: u32) -> UVec2:
        return UVec2(v, v)

    pub fn dot(self, o: UVec2) -> u32:
        return self.x * o.x + self.y * o.y

    ## The smaller of each pair of components.
    pub fn min(self, o: UVec2) -> UVec2:
        return UVec2(min(self.x, o.x), min(self.y, o.y))

    ## The larger of each pair of components.
    pub fn max(self, o: UVec2) -> UVec2:
        return UVec2(max(self.x, o.x), max(self.y, o.y))

    ## This vector with a z component.
    pub fn extend(self, z: u32) -> UVec3:
        return UVec3(self.x, self.y, z)

extend UVec2 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec2

    fn add(self, o: UVec2) -> UVec2:
        return UVec2(self.x + o.x, self.y + o.y)

    fn sub(self, o: UVec2) -> UVec2:
        return UVec2(self.x - o.x, self.y - o.y)

    fn mul(self, o: UVec2) -> UVec2:
        return UVec2(self.x * o.x, self.y * o.y)

    fn floordiv(self, o: UVec2) -> UVec2:
        return UVec2(self.x // o.x, self.y // o.y)

    fn rem(self, o: UVec2) -> UVec2:
        return UVec2(self.x % o.x, self.y % o.y)

extend UVec2 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec2

    fn add(self, s: u32) -> UVec2:
        return UVec2(self.x + s, self.y + s)

    fn sub(self, s: u32) -> UVec2:
        return UVec2(self.x - s, self.y - s)

    fn mul(self, s: u32) -> UVec2:
        return UVec2(self.x * s, self.y * s)

    fn floordiv(self, s: u32) -> UVec2:
        return UVec2(self.x // s, self.y // s)

    fn rem(self, s: u32) -> UVec2:
        return UVec2(self.x % s, self.y % s)

extend u32 implements Add[UVec2], Sub[UVec2], Mul[UVec2], FloorDiv[UVec2], Rem[UVec2]:
    type Output = UVec2

    fn add(self, v: UVec2) -> UVec2:
        return UVec2(self + v.x, self + v.y)

    fn sub(self, v: UVec2) -> UVec2:
        return UVec2(self - v.x, self - v.y)

    fn mul(self, v: UVec2) -> UVec2:
        return UVec2(self * v.x, self * v.y)

    fn floordiv(self, v: UVec2) -> UVec2:
        return UVec2(self // v.x, self // v.y)

    fn rem(self, v: UVec2) -> UVec2:
        return UVec2(self % v.x, self % v.y)

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
    pub fn splat(v: u32) -> UVec3:
        return UVec3(v, v, v)

    pub fn dot(self, o: UVec3) -> u32:
        return self.x * o.x + self.y * o.y + self.z * o.z

    ## The smaller of each pair of components.
    pub fn min(self, o: UVec3) -> UVec3:
        return UVec3(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z))

    ## The larger of each pair of components.
    pub fn max(self, o: UVec3) -> UVec3:
        return UVec3(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z))

    ## This vector with a w component.
    pub fn extend(self, w: u32) -> UVec4:
        return UVec4(self.x, self.y, self.z, w)

    ## This vector without its last component.
    pub fn truncate(self) -> UVec2:
        return UVec2(self.x, self.y)

extend UVec3 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec3

    fn add(self, o: UVec3) -> UVec3:
        return UVec3(self.x + o.x, self.y + o.y, self.z + o.z)

    fn sub(self, o: UVec3) -> UVec3:
        return UVec3(self.x - o.x, self.y - o.y, self.z - o.z)

    fn mul(self, o: UVec3) -> UVec3:
        return UVec3(self.x * o.x, self.y * o.y, self.z * o.z)

    fn floordiv(self, o: UVec3) -> UVec3:
        return UVec3(self.x // o.x, self.y // o.y, self.z // o.z)

    fn rem(self, o: UVec3) -> UVec3:
        return UVec3(self.x % o.x, self.y % o.y, self.z % o.z)

extend UVec3 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec3

    fn add(self, s: u32) -> UVec3:
        return UVec3(self.x + s, self.y + s, self.z + s)

    fn sub(self, s: u32) -> UVec3:
        return UVec3(self.x - s, self.y - s, self.z - s)

    fn mul(self, s: u32) -> UVec3:
        return UVec3(self.x * s, self.y * s, self.z * s)

    fn floordiv(self, s: u32) -> UVec3:
        return UVec3(self.x // s, self.y // s, self.z // s)

    fn rem(self, s: u32) -> UVec3:
        return UVec3(self.x % s, self.y % s, self.z % s)

extend u32 implements Add[UVec3], Sub[UVec3], Mul[UVec3], FloorDiv[UVec3], Rem[UVec3]:
    type Output = UVec3

    fn add(self, v: UVec3) -> UVec3:
        return UVec3(self + v.x, self + v.y, self + v.z)

    fn sub(self, v: UVec3) -> UVec3:
        return UVec3(self - v.x, self - v.y, self - v.z)

    fn mul(self, v: UVec3) -> UVec3:
        return UVec3(self * v.x, self * v.y, self * v.z)

    fn floordiv(self, v: UVec3) -> UVec3:
        return UVec3(self // v.x, self // v.y, self // v.z)

    fn rem(self, v: UVec3) -> UVec3:
        return UVec3(self % v.x, self % v.y, self % v.z)

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
    pub fn splat(v: u32) -> UVec4:
        return UVec4(v, v, v, v)

    pub fn dot(self, o: UVec4) -> u32:
        return self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w

    ## The smaller of each pair of components.
    pub fn min(self, o: UVec4) -> UVec4:
        return UVec4(min(self.x, o.x), min(self.y, o.y), min(self.z, o.z), min(self.w, o.w))

    ## The larger of each pair of components.
    pub fn max(self, o: UVec4) -> UVec4:
        return UVec4(max(self.x, o.x), max(self.y, o.y), max(self.z, o.z), max(self.w, o.w))

    ## This vector without its last component.
    pub fn truncate(self) -> UVec3:
        return UVec3(self.x, self.y, self.z)

extend UVec4 implements Add, Sub, Mul, FloorDiv, Rem:
    type Output = UVec4

    fn add(self, o: UVec4) -> UVec4:
        return UVec4(self.x + o.x, self.y + o.y, self.z + o.z, self.w + o.w)

    fn sub(self, o: UVec4) -> UVec4:
        return UVec4(self.x - o.x, self.y - o.y, self.z - o.z, self.w - o.w)

    fn mul(self, o: UVec4) -> UVec4:
        return UVec4(self.x * o.x, self.y * o.y, self.z * o.z, self.w * o.w)

    fn floordiv(self, o: UVec4) -> UVec4:
        return UVec4(self.x // o.x, self.y // o.y, self.z // o.z, self.w // o.w)

    fn rem(self, o: UVec4) -> UVec4:
        return UVec4(self.x % o.x, self.y % o.y, self.z % o.z, self.w % o.w)

extend UVec4 implements Add[u32], Sub[u32], Mul[u32], FloorDiv[u32], Rem[u32]:
    type Output = UVec4

    fn add(self, s: u32) -> UVec4:
        return UVec4(self.x + s, self.y + s, self.z + s, self.w + s)

    fn sub(self, s: u32) -> UVec4:
        return UVec4(self.x - s, self.y - s, self.z - s, self.w - s)

    fn mul(self, s: u32) -> UVec4:
        return UVec4(self.x * s, self.y * s, self.z * s, self.w * s)

    fn floordiv(self, s: u32) -> UVec4:
        return UVec4(self.x // s, self.y // s, self.z // s, self.w // s)

    fn rem(self, s: u32) -> UVec4:
        return UVec4(self.x % s, self.y % s, self.z % s, self.w % s)

extend u32 implements Add[UVec4], Sub[UVec4], Mul[UVec4], FloorDiv[UVec4], Rem[UVec4]:
    type Output = UVec4

    fn add(self, v: UVec4) -> UVec4:
        return UVec4(self + v.x, self + v.y, self + v.z, self + v.w)

    fn sub(self, v: UVec4) -> UVec4:
        return UVec4(self - v.x, self - v.y, self - v.z, self - v.w)

    fn mul(self, v: UVec4) -> UVec4:
        return UVec4(self * v.x, self * v.y, self * v.z, self * v.w)

    fn floordiv(self, v: UVec4) -> UVec4:
        return UVec4(self // v.x, self // v.y, self // v.z, self // v.w)

    fn rem(self, v: UVec4) -> UVec4:
        return UVec4(self % v.x, self % v.y, self % v.z, self % v.w)


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

    pub fn from_cols(x_axis: Vec2, y_axis: Vec2) -> Mat2:
        return Mat2(x_axis, y_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub fn from_diagonal(d: Vec2) -> Mat2:
        return Mat2(Vec2(d.x, 0.0), Vec2(0.0, d.y))

    ## A rotation by `angle` radians, counterclockwise.
    pub fn from_angle(angle: f32) -> Mat2:
        s = angle.sin()
        c = angle.cos()
        return Mat2(Vec2(c, s), Vec2(-s, c))

    ## Column `i`, 0 or 1; another `i` panics.
    pub fn col(self, i: int) -> Vec2:
        if i == 0:
            return self.x_axis
        if i == 1:
            return self.y_axis
        panic(f"a Mat2 has no column {i}")

    ## Row `i`, 0 or 1; another `i` panics.
    pub fn row(self, i: int) -> Vec2:
        if i == 0:
            return Vec2(self.x_axis.x, self.y_axis.x)
        if i == 1:
            return Vec2(self.x_axis.y, self.y_axis.y)
        panic(f"a Mat2 has no row {i}")

    pub fn transpose(self) -> Mat2:
        return Mat2(self.row(0), self.row(1))

    pub fn determinant(self) -> f32:
        return self.x_axis.x * self.y_axis.y - self.y_axis.x * self.x_axis.y

    ## The inverse, or `None` for a matrix with none (its determinant zero).
    pub fn try_inverse(self) -> Option[Mat2]:
        det = self.determinant()
        if det == 0.0:
            return None
        inv = 1.0 / det
        return Some(Mat2(Vec2(self.y_axis.y * inv, -self.x_axis.y * inv), Vec2(-self.y_axis.x * inv, self.x_axis.x * inv)))

    ## The inverse; a matrix with none panics.
    pub fn inverse(self) -> Mat2:
        match self.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

extend Mat2 implements Add, Sub, Mul, Neg:
    type Output = Mat2

    fn add(self, o: Mat2) -> Mat2:
        return Mat2(self.x_axis + o.x_axis, self.y_axis + o.y_axis)

    fn sub(self, o: Mat2) -> Mat2:
        return Mat2(self.x_axis - o.x_axis, self.y_axis - o.y_axis)

    fn mul(self, o: Mat2) -> Mat2:
        return Mat2(self * o.x_axis, self * o.y_axis)

    fn neg(self) -> Mat2:
        return Mat2(-self.x_axis, -self.y_axis)

extend Mat2 implements Mul[Vec2]:
    type Output = Vec2

    fn mul(self, v: Vec2) -> Vec2:
        return self.y_axis.mul_add(Vec2.splat(v.y), self.x_axis * v.x)

extend Mat2 implements Mul[f32]:
    type Output = Mat2

    fn mul(self, s: f32) -> Mat2:
        return Mat2(self.x_axis * s, self.y_axis * s)

extend f32 implements Mul[Mat2]:
    type Output = Mat2

    fn mul(self, m: Mat2) -> Mat2:
        return m * self

## A 3×3 matrix: a rotation or scale in space.
@derive(Copy)
@layout(c)
pub struct Mat3:
    pub x_axis: Vec3
    pub y_axis: Vec3
    pub z_axis: Vec3

    pub const ZERO: Mat3 = Mat3(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 0.0))
    pub const IDENTITY: Mat3 = Mat3(Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0), Vec3(0.0, 0.0, 1.0))

    pub fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Mat3:
        return Mat3(x_axis, y_axis, z_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub fn from_diagonal(d: Vec3) -> Mat3:
        return Mat3(Vec3(d.x, 0.0, 0.0), Vec3(0.0, d.y, 0.0), Vec3(0.0, 0.0, d.z))

    ## A scale by `s` along each axis.
    pub fn from_scale(s: Vec3) -> Mat3:
        return Mat3.from_diagonal(s)

    ## A rotation by `angle` radians about the z axis, counterclockwise
    ## looking down it.
    pub fn from_angle(angle: f32) -> Mat3:
        s = angle.sin()
        c = angle.cos()
        return Mat3(Vec3(c, s, 0.0), Vec3(-s, c, 0.0), Vec3(0.0, 0.0, 1.0))

    ## The rotation `q` is, for a `q` of length one.
    pub fn from_quat(q: Quat) -> Mat3:
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

    ## Column `i`, 0 to 2; another `i` panics.
    pub fn col(self, i: int) -> Vec3:
        if i == 0:
            return self.x_axis
        if i == 1:
            return self.y_axis
        if i == 2:
            return self.z_axis
        panic(f"a Mat3 has no column {i}")

    ## Row `i`, 0 to 2; another `i` panics.
    pub fn row(self, i: int) -> Vec3:
        if i == 0:
            return Vec3(self.x_axis.x, self.y_axis.x, self.z_axis.x)
        if i == 1:
            return Vec3(self.x_axis.y, self.y_axis.y, self.z_axis.y)
        if i == 2:
            return Vec3(self.x_axis.z, self.y_axis.z, self.z_axis.z)
        panic(f"a Mat3 has no row {i}")

    pub fn transpose(self) -> Mat3:
        return Mat3(self.row(0), self.row(1), self.row(2))

    pub fn determinant(self) -> f32:
        return self.x_axis.dot(self.y_axis.cross(self.z_axis))

    ## The inverse, or `None` for a matrix with none (its determinant zero).
    ## Its rows are the cross products of the columns, over the determinant.
    pub fn try_inverse(self) -> Option[Mat3]:
        yz = self.y_axis.cross(self.z_axis)
        zx = self.z_axis.cross(self.x_axis)
        xy = self.x_axis.cross(self.y_axis)
        det = self.x_axis.dot(yz)
        if det == 0.0:
            return None
        return Some(Mat3(yz, zx, xy).transpose() * (1.0 / det))

    ## The inverse; a matrix with none panics.
    pub fn inverse(self) -> Mat3:
        match self.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

extend Mat3 implements Add, Sub, Mul, Neg:
    type Output = Mat3

    fn add(self, o: Mat3) -> Mat3:
        return Mat3(self.x_axis + o.x_axis, self.y_axis + o.y_axis, self.z_axis + o.z_axis)

    fn sub(self, o: Mat3) -> Mat3:
        return Mat3(self.x_axis - o.x_axis, self.y_axis - o.y_axis, self.z_axis - o.z_axis)

    fn mul(self, o: Mat3) -> Mat3:
        return Mat3(self * o.x_axis, self * o.y_axis, self * o.z_axis)

    fn neg(self) -> Mat3:
        return Mat3(-self.x_axis, -self.y_axis, -self.z_axis)

extend Mat3 implements Mul[Vec3]:
    type Output = Vec3

    fn mul(self, v: Vec3) -> Vec3:
        r = self.y_axis.mul_add(Vec3.splat(v.y), self.x_axis * v.x)
        return self.z_axis.mul_add(Vec3.splat(v.z), r)

extend Mat3 implements Mul[f32]:
    type Output = Mat3

    fn mul(self, s: f32) -> Mat3:
        return Mat3(self.x_axis * s, self.y_axis * s, self.z_axis * s)

extend f32 implements Mul[Mat3]:
    type Output = Mat3

    fn mul(self, m: Mat3) -> Mat3:
        return m * self

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

    pub fn from_cols(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Mat4:
        return Mat4(x_axis, y_axis, z_axis, w_axis)

    ## `d` on the diagonal and zeros elsewhere.
    pub fn from_diagonal(d: Vec4) -> Mat4:
        return Mat4(Vec4(d.x, 0.0, 0.0, 0.0), Vec4(0.0, d.y, 0.0, 0.0), Vec4(0.0, 0.0, d.z, 0.0), Vec4(0.0, 0.0, 0.0, d.w))

    ## A move by `t`.
    pub fn from_translation(t: Vec3) -> Mat4:
        return Mat4(Vec4(1.0, 0.0, 0.0, 0.0), Vec4(0.0, 1.0, 0.0, 0.0), Vec4(0.0, 0.0, 1.0, 0.0), t.extend(1.0))

    ## A scale by `s` along each axis.
    pub fn from_scale(s: Vec3) -> Mat4:
        return Mat4.from_diagonal(s.extend(1.0))

    ## The rotation `q` is, for a `q` of length one.
    pub fn from_quat(q: Quat) -> Mat4:
        m = Mat3.from_quat(q)
        return Mat4(m.x_axis.extend(0.0), m.y_axis.extend(0.0), m.z_axis.extend(0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the x axis.
    pub fn from_rotation_x(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(1.0, 0.0, 0.0, 0.0), Vec4(0.0, c, s, 0.0), Vec4(0.0, -s, c, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the y axis.
    pub fn from_rotation_y(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(c, 0.0, -s, 0.0), Vec4(0.0, 1.0, 0.0, 0.0), Vec4(s, 0.0, c, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## A rotation by `angle` radians about the z axis.
    pub fn from_rotation_z(angle: f32) -> Mat4:
        s = angle.sin()
        c = angle.cos()
        return Mat4(Vec4(c, s, 0.0, 0.0), Vec4(-s, c, 0.0, 0.0), Vec4(0.0, 0.0, 1.0, 0.0), Vec4(0.0, 0.0, 0.0, 1.0))

    ## Scale by `scale`, then rotate by `rotation`, then move by
    ## `translation`.
    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Mat4:
        m = Mat3.from_quat(rotation)
        return Mat4(
            (m.x_axis * scale.x).extend(0.0),
            (m.y_axis * scale.y).extend(0.0),
            (m.z_axis * scale.z).extend(0.0),
            translation.extend(1.0),
        )

    ## A right-handed view from `eye` towards `center`, with `up` the rough
    ## direction of the view's y axis: the camera looks down its −z axis.
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Mat4:
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
    pub fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4:
        h = 1.0 / (fov_y * 0.5).tan()
        w = h / aspect
        r = far / (near - far)
        return Mat4(Vec4(w, 0.0, 0.0, 0.0), Vec4(0.0, h, 0.0, 0.0), Vec4(0.0, 0.0, r, -1.0), Vec4(0.0, 0.0, r * near, 0.0))

    ## A right-handed orthographic projection with depth in `[0, 1]`.
    pub fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4:
        rw = 1.0 / (right - left)
        rh = 1.0 / (top - bottom)
        r = 1.0 / (near - far)
        return Mat4(
            Vec4(rw + rw, 0.0, 0.0, 0.0),
            Vec4(0.0, rh + rh, 0.0, 0.0),
            Vec4(0.0, 0.0, r, 0.0),
            Vec4(-(left + right) * rw, -(top + bottom) * rh, r * near, 1.0),
        )

    ## Column `i`, 0 to 3; another `i` panics.
    pub fn col(self, i: int) -> Vec4:
        if i == 0:
            return self.x_axis
        if i == 1:
            return self.y_axis
        if i == 2:
            return self.z_axis
        if i == 3:
            return self.w_axis
        panic(f"a Mat4 has no column {i}")

    ## Row `i`, 0 to 3; another `i` panics.
    pub fn row(self, i: int) -> Vec4:
        if i == 0:
            return Vec4(self.x_axis.x, self.y_axis.x, self.z_axis.x, self.w_axis.x)
        if i == 1:
            return Vec4(self.x_axis.y, self.y_axis.y, self.z_axis.y, self.w_axis.y)
        if i == 2:
            return Vec4(self.x_axis.z, self.y_axis.z, self.z_axis.z, self.w_axis.z)
        if i == 3:
            return Vec4(self.x_axis.w, self.y_axis.w, self.z_axis.w, self.w_axis.w)
        panic(f"a Mat4 has no row {i}")

    pub fn transpose(self) -> Mat4:
        return Mat4(self.row(0), self.row(1), self.row(2), self.row(3))

    pub fn determinant(self) -> f32:
        m = MinorPairs.of(self)
        return m.determinant()

    ## The inverse, or `None` for a matrix with none (its determinant zero):
    ## the adjugate over the determinant, from the 2×2 minors of the top and
    ## bottom halves.
    pub fn try_inverse(self) -> Option[Mat4]:
        m = MinorPairs.of(self)
        det = m.determinant()
        if det == 0.0:
            return None
        a = self.row(0)
        b = self.row(1)
        c = self.row(2)
        d = self.row(3)
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
    pub fn inverse(self) -> Mat4:
        match self.try_inverse():
            Some(m):
                return m
            None:
                panic("inverse of a singular matrix; use try_inverse")

    ## The point `p` transformed: `p` with `w` 1, divided by the result's
    ## `w` (1 for any matrix that is not a projection).
    pub fn transform_point3(self, p: Vec3) -> Vec3:
        r = self * p.extend(1.0)
        return r.truncate() / r.w

    ## The direction `v` transformed: `v` with `w` 0, so no translation.
    pub fn transform_vector3(self, v: Vec3) -> Vec3:
        return (self * v.extend(0.0)).truncate()

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

    fn of(m: Mat4) -> MinorPairs:
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

    fn determinant(self) -> f32:
        return self.s0 * self.c5 - self.s1 * self.c4 + self.s2 * self.c3 + self.s3 * self.c2 - self.s4 * self.c1 + self.s5 * self.c0

extend Mat4 implements Add, Sub, Mul, Neg:
    type Output = Mat4

    fn add(self, o: Mat4) -> Mat4:
        return Mat4(self.x_axis + o.x_axis, self.y_axis + o.y_axis, self.z_axis + o.z_axis, self.w_axis + o.w_axis)

    fn sub(self, o: Mat4) -> Mat4:
        return Mat4(self.x_axis - o.x_axis, self.y_axis - o.y_axis, self.z_axis - o.z_axis, self.w_axis - o.w_axis)

    fn mul(self, o: Mat4) -> Mat4:
        return Mat4(self * o.x_axis, self * o.y_axis, self * o.z_axis, self * o.w_axis)

    fn neg(self) -> Mat4:
        return Mat4(-self.x_axis, -self.y_axis, -self.z_axis, -self.w_axis)

extend Mat4 implements Mul[Vec4]:
    type Output = Vec4

    fn mul(self, v: Vec4) -> Vec4:
        r = self.y_axis.mul_add(Vec4.splat(v.y), self.x_axis * v.x)
        r = self.z_axis.mul_add(Vec4.splat(v.z), r)
        return self.w_axis.mul_add(Vec4.splat(v.w), r)

extend Mat4 implements Mul[f32]:
    type Output = Mat4

    fn mul(self, s: f32) -> Mat4:
        return Mat4(self.x_axis * s, self.y_axis * s, self.z_axis * s, self.w_axis * s)

extend f32 implements Mul[Mat4]:
    type Output = Mat4

    fn mul(self, m: Mat4) -> Mat4:
        return m * self

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
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat:
        half = angle * 0.5
        s = half.sin()
        return Quat(axis.x * s, axis.y * s, axis.z * s, half.cos())

    pub fn from_rotation_x(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(1.0, 0.0, 0.0), angle)

    pub fn from_rotation_y(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(0.0, 1.0, 0.0), angle)

    pub fn from_rotation_z(angle: f32) -> Quat:
        return Quat.from_axis_angle(Vec3(0.0, 0.0, 1.0), angle)

    pub fn dot(self, o: Quat) -> f32:
        return self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w

    pub fn length(self) -> f32:
        return self.dot(self).sqrt()

    ## The same rotation, of length one; a zero quaternion panics.
    pub fn normalize(self) -> Quat:
        length = self.length()
        if length == 0.0:
            panic("normalize of a zero quaternion")
        inv = 1.0 / length
        return Quat(self.x * inv, self.y * inv, self.z * inv, self.w * inv)

    ## The opposite rotation, for a quaternion of length one.
    pub fn conjugate(self) -> Quat:
        return Quat(-self.x, -self.y, -self.z, self.w)

    ## The opposite rotation, for a quaternion of any length but zero.
    pub fn inverse(self) -> Quat:
        inv = 1.0 / self.dot(self)
        return Quat(-self.x * inv, -self.y * inv, -self.z * inv, self.w * inv)

    ## The rotation a fraction `t` of the way from this one to `end`, along
    ## the shorter arc at a steady speed.
    pub fn slerp(self, end: Quat, t: f32) -> Quat:
        cos_theta = self.dot(end)
        e = end
        if cos_theta < 0.0:
            e = Quat(-end.x, -end.y, -end.z, -end.w)
            cos_theta = -cos_theta
        # Nearly one rotation: the arc is too short to measure, so a straight
        # line serves.
        if cos_theta > 0.9995:
            return Quat(
                self.x + (e.x - self.x) * t,
                self.y + (e.y - self.y) * t,
                self.z + (e.z - self.z) * t,
                self.w + (e.w - self.w) * t,
            ).normalize()
        theta = cos_theta.acos()
        sin_theta = theta.sin()
        a = ((1.0 - t) * theta).sin() / sin_theta
        b = (t * theta).sin() / sin_theta
        return Quat(self.x * a + e.x * b, self.y * a + e.y * b, self.z * a + e.z * b, self.w * a + e.w * b)

extend Quat implements Mul:
    type Output = Quat

    fn mul(self, o: Quat) -> Quat:
        return Quat(
            self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
            self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
        )

extend Quat implements Mul[Vec3]:
    type Output = Vec3

    ## `v` rotated: `v + w t + q × t` with `t = 2 q × v`, `q` the vector part,
    ## `w t` added with one rounding (`[STD-3]`).
    fn mul(self, v: Vec3) -> Vec3:
        q = Vec3(self.x, self.y, self.z)
        t = q.cross(v) * 2.0
        return t.mul_add(Vec3.splat(self.w), v + q.cross(t))

## A scale, then a rotation, then a move.
@derive(Copy)
@layout(c)
pub struct Transform:
    pub translation: Vec3
    pub rotation: Quat
    pub scale: Vec3

    pub const IDENTITY: Transform = Transform(Vec3(0.0, 0.0, 0.0), Quat(0.0, 0.0, 0.0, 1.0), Vec3(1.0, 1.0, 1.0))

    pub fn from_translation(t: Vec3) -> Transform:
        return Transform(t, Quat(0.0, 0.0, 0.0, 1.0), Vec3(1.0, 1.0, 1.0))

    pub fn from_rotation(q: Quat) -> Transform:
        return Transform(Vec3(0.0, 0.0, 0.0), q, Vec3(1.0, 1.0, 1.0))

    pub fn from_scale(s: Vec3) -> Transform:
        return Transform(Vec3(0.0, 0.0, 0.0), Quat(0.0, 0.0, 0.0, 1.0), s)

    ## The point `p`, scaled, rotated and moved.
    pub fn transform_point(self, p: Vec3) -> Vec3:
        return self.rotation * (self.scale * p) + self.translation

    ## The direction `v`, scaled and rotated but not moved.
    pub fn transform_vector(self, v: Vec3) -> Vec3:
        return self.rotation * (self.scale * v)

    pub fn to_mat4(self) -> Mat4:
        return Mat4.from_scale_rotation_translation(self.scale, self.rotation, self.translation)

extend Transform implements Mul:
    type Output = Transform

    ## `o`, then this one. With a scale that is not the same along every
    ## axis and a rotation, the result is the usual approximation: scales
    ## multiply axis by axis.
    fn mul(self, o: Transform) -> Transform:
        return Transform(self.transform_point(o.translation), self.rotation * o.rotation, self.scale * o.scale)

## -- shapes (`[STD-28]`, ODR-043) ----------------------------------------------

## An axis-aligned box: the points between `min` and `max` in every axis.
@derive(Copy)
@layout(c)
pub struct Aabb:
    pub min: Vec3
    pub max: Vec3

    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Aabb:
        return Aabb(center - half_extents, center + half_extents)

    pub fn center(self) -> Vec3:
        return (self.min + self.max) * 0.5

    pub fn half_extents(self) -> Vec3:
        return (self.max - self.min) * 0.5

    ## Whether `p` is inside, or on the boundary.
    pub fn contains(self, p: Vec3) -> bool:
        return self.min.x <= p.x and p.x <= self.max.x and self.min.y <= p.y and p.y <= self.max.y and self.min.z <= p.z and p.z <= self.max.z

    ## Whether the two boxes share a point.
    pub fn intersects(self, o: Aabb) -> bool:
        return self.min.x <= o.max.x and o.min.x <= self.max.x and self.min.y <= o.max.y and o.min.y <= self.max.y and self.min.z <= o.max.z and o.min.z <= self.max.z

    ## The least box holding both.
    pub fn union(self, o: Aabb) -> Aabb:
        return Aabb(self.min.min(o.min), self.max.max(o.max))

    ## The least box holding this one and `p`.
    pub fn expand(self, p: Vec3) -> Aabb:
        return Aabb(self.min.min(p), self.max.max(p))

## A ball: the points no further than `radius` from `center`.
@derive(Copy)
@layout(c)
pub struct Sphere:
    pub center: Vec3
    pub radius: f32

    pub fn contains(self, p: Vec3) -> bool:
        return (p - self.center).length_squared() <= self.radius * self.radius

    pub fn intersects(self, o: Sphere) -> bool:
        reach = self.radius + o.radius
        return (o.center - self.center).length_squared() <= reach * reach

    ## Whether the ball and the box share a point: the box's point nearest
    ## the center is within `radius`.
    pub fn intersects_aabb(self, b: Aabb) -> bool:
        nearest = self.center.max(b.min).min(b.max)
        return (nearest - self.center).length_squared() <= self.radius * self.radius

## A half-line: `origin + direction * t` for every `t ≥ 0`.
@derive(Copy)
@layout(c)
pub struct Ray:
    pub origin: Vec3
    pub direction: Vec3

    ## The point at `t`.
    pub fn at(self, t: f32) -> Vec3:
        return self.origin + self.direction * t

    ## The least `t ≥ 0` at which the ray is on the plane, or `None`: 0 when
    ## it starts on it.
    pub fn intersect_plane(self, p: Plane) -> Option[f32]:
        distance = p.signed_distance(self.origin)
        if distance == 0.0:
            return Some(0.0)
        facing = p.normal.dot(self.direction)
        if facing == 0.0:
            return None
        t = -distance / facing
        if t >= 0.0:
            return Some(t)
        return None

    ## The least `t ≥ 0` at which the ray is in the ball, or `None`: 0 when
    ## it starts inside.
    pub fn intersect_sphere(self, s: Sphere) -> Option[f32]:
        to_origin = self.origin - s.center
        c = to_origin.length_squared() - s.radius * s.radius
        if c <= 0.0:
            return Some(0.0)
        a = self.direction.length_squared()
        if a == 0.0:
            return None
        b = to_origin.dot(self.direction)
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
    pub fn intersect_aabb(self, b: Aabb) -> Option[f32]:
        low: f32 = 0.0
        high = f32.INF
        (l, h) = slab(self.origin.x, self.direction.x, b.min.x, b.max.x)
        low = max(low, l)
        high = min(high, h)
        (l, h) = slab(self.origin.y, self.direction.y, b.min.y, b.max.y)
        low = max(low, l)
        high = min(high, h)
        (l, h) = slab(self.origin.z, self.direction.z, b.min.z, b.max.z)
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
    pub fn from_point_normal(p: Vec3, normal: Vec3) -> Plane:
        n = normal.normalize()
        return Plane(n, -n.dot(p))

    ## The plane through three points, facing the side from which they run
    ## counterclockwise.
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Plane:
        n = (b - a).cross(c - a).normalize()
        return Plane(n, -n.dot(a))

    pub fn signed_distance(self, p: Vec3) -> f32:
        return self.normal.dot(p) + self.d

    ## The same plane, with a normal of length one.
    pub fn normalize(self) -> Plane:
        length = self.normal.length()
        if length == 0.0:
            panic("normalize of a plane with no normal")
        return Plane(self.normal / length, self.d / length)

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
    pub fn from_view_projection(m: Mat4) -> Frustum:
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

    pub fn contains_point(self, p: Vec3) -> bool:
        return self.left.signed_distance(p) >= 0.0 and self.right.signed_distance(p) >= 0.0 and self.bottom.signed_distance(p) >= 0.0 and self.top.signed_distance(p) >= 0.0 and self.near.signed_distance(p) >= 0.0 and self.far.signed_distance(p) >= 0.0

    ## Whether the ball may be seen: no plane has it wholly outside.
    pub fn intersects_sphere(self, s: Sphere) -> bool:
        r = -s.radius
        return self.left.signed_distance(s.center) >= r and self.right.signed_distance(s.center) >= r and self.bottom.signed_distance(s.center) >= r and self.top.signed_distance(s.center) >= r and self.near.signed_distance(s.center) >= r and self.far.signed_distance(s.center) >= r

    ## Whether the box may be seen: no plane has its corner furthest along
    ## the plane's normal outside.
    pub fn intersects_aabb(self, b: Aabb) -> bool:
        return reaches(self.left, b) and reaches(self.right, b) and reaches(self.bottom, b) and reaches(self.top, b) and reaches(self.near, b) and reaches(self.far, b)

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
    pub fn new() -> KahanSum:
        return KahanSum()

    pub fn add(mut self, x: f64):
        t = self.sum + x
        if abs(self.sum) >= abs(x):
            self.lost += (self.sum - t) + x
        else:
            self.lost += (x - t) + self.sum
        self.sum = t

    pub fn value(self) -> f64:
        return self.sum + self.lost

    ## The sum of `xs`, compensated.
    pub fn of(xs: Span[f64]) -> f64:
        k = KahanSum.new()
        for x in xs:
            k.add(x)
        return k.value()
