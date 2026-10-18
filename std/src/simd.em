## `std.simd` — vectors of lanes, operated on together (Part XII.2).
##
## `[SIMD-1]` — `f32x4`, `f32x8`, `f32x16`, `f64x2`, `f64x4`, `f64x8`, the signed and unsigned
## integer vectors of 8 to 64-bit lanes (`i8x16` .. `u64x8`: 128, 256 and 512 bits of each lane
## width), and one mask type per lane width and count (`mask32x8` for `f32x8`, `i32x8` and
## `u32x8`). Each is a `Copy` value aligned to its size; `V.LANES` is its lane count, and
## `native_lanes[T]()` the target's preferred count for `T`. The C backend lowers them through
## `ember_simd.h`: the target's vector instructions (SSE2 on x86-64, NEON on AArch64) where it has
## them, the same operations lane by lane elsewhere.
##
## The compiler knows these types by their names here, as it knows `std.sync`'s. What it gives
## them (`[SIMD-4]`, `[SIMD-8]`, `[SIMD-9]`):
##
## - `V.splat(x)`, `V.from_array(a)` and `v.to_array()` (written here), `V.load(s)` and `v.store(s)`
##   (`s.len()` at least `V.LANES`), `V.gather(s, idx)` and `v.scatter(s, idx)` (`idx` an integer
##   vector of as many lanes, each lane's index checked against `s.len()`): a failure is a `Bounds`
##   panic naming the lane. The unchecked forms `load_unchecked`, `store_unchecked`,
##   `gather_unchecked` and `scatter_unchecked` are `unsafe`.
## - Lane-wise operators, following the scalar rules: `+`, `-`, `*` and unary `-` on every vector,
##   `/` on float vectors, `&`, `|`, `^` on integer vectors and masks. Integer lanes panic on
##   overflow (`[TYP-8]`) unless the function is `@overflow(wrap)` (or `saturate`), or the code
##   calls `wrapping_add`, `wrapping_sub`, `wrapping_mul`; float lanes are IEEE with no
##   contraction (`[TYP-9]`). `==`, `!=`, `<`, `<=`, `>`, `>=` compare lane by lane into a mask.
## - `reduce_add`, `reduce_min`, `reduce_max` (lane 0 first, then each next lane in order, so the
##   answer is the same on every target), `shuffle([…])` with a constant list of lane numbers,
##   `fma(b, c)` (`self * b + c` rounded once), `min`, `max`, `abs`, `sqrt`, and the approximations
##   `rsqrt_approx` and `rcp_approx` (whose last bits depend on the target).
## - On a mask: `any()`, `all()`, `select(a, b)` (each lane from `a` where the mask is set, else
##   from `b`), `&`, `|`, `^`, `M.splat(b)` and `to_array()`.

## `[SIMD-1]` — the target's preferred lane count for `T`: its vector registers' width over `T`'s
## size. Every target of the C backend (x86-64 with SSE2, AArch64 with NEON) has 16-byte registers.
pub fn native_lanes[T]() -> int:
    return (16 // size_of[T]()) as int

## 4 `f32` lanes, 16 bytes (float).
@derive(Copy)
pub struct f32x4:
    lanes: [f32; 4]

    pub const LANES: int = 4

extend f32x4:
    pub fn from_array(lanes: [f32; 4]) -> f32x4:
        return f32x4(lanes=lanes)

    pub fn to_array(self) -> [f32; 4]:
        return self.lanes

## 8 `f32` lanes, 32 bytes (float).
@derive(Copy)
pub struct f32x8:
    lanes: [f32; 8]

    pub const LANES: int = 8

extend f32x8:
    pub fn from_array(lanes: [f32; 8]) -> f32x8:
        return f32x8(lanes=lanes)

    pub fn to_array(self) -> [f32; 8]:
        return self.lanes

## 16 `f32` lanes, 64 bytes (float).
@derive(Copy)
pub struct f32x16:
    lanes: [f32; 16]

    pub const LANES: int = 16

extend f32x16:
    pub fn from_array(lanes: [f32; 16]) -> f32x16:
        return f32x16(lanes=lanes)

    pub fn to_array(self) -> [f32; 16]:
        return self.lanes

## 2 `f64` lanes, 16 bytes (float).
@derive(Copy)
pub struct f64x2:
    lanes: [f64; 2]

    pub const LANES: int = 2

extend f64x2:
    pub fn from_array(lanes: [f64; 2]) -> f64x2:
        return f64x2(lanes=lanes)

    pub fn to_array(self) -> [f64; 2]:
        return self.lanes

## 4 `f64` lanes, 32 bytes (float).
@derive(Copy)
pub struct f64x4:
    lanes: [f64; 4]

    pub const LANES: int = 4

extend f64x4:
    pub fn from_array(lanes: [f64; 4]) -> f64x4:
        return f64x4(lanes=lanes)

    pub fn to_array(self) -> [f64; 4]:
        return self.lanes

## 8 `f64` lanes, 64 bytes (float).
@derive(Copy)
pub struct f64x8:
    lanes: [f64; 8]

    pub const LANES: int = 8

extend f64x8:
    pub fn from_array(lanes: [f64; 8]) -> f64x8:
        return f64x8(lanes=lanes)

    pub fn to_array(self) -> [f64; 8]:
        return self.lanes

## 16 `i8` lanes, 16 bytes (signed integer).
@derive(Copy)
pub struct i8x16:
    lanes: [i8; 16]

    pub const LANES: int = 16

extend i8x16:
    pub fn from_array(lanes: [i8; 16]) -> i8x16:
        return i8x16(lanes=lanes)

    pub fn to_array(self) -> [i8; 16]:
        return self.lanes

## 32 `i8` lanes, 32 bytes (signed integer).
@derive(Copy)
pub struct i8x32:
    lanes: [i8; 32]

    pub const LANES: int = 32

extend i8x32:
    pub fn from_array(lanes: [i8; 32]) -> i8x32:
        return i8x32(lanes=lanes)

    pub fn to_array(self) -> [i8; 32]:
        return self.lanes

## 64 `i8` lanes, 64 bytes (signed integer).
@derive(Copy)
pub struct i8x64:
    lanes: [i8; 64]

    pub const LANES: int = 64

extend i8x64:
    pub fn from_array(lanes: [i8; 64]) -> i8x64:
        return i8x64(lanes=lanes)

    pub fn to_array(self) -> [i8; 64]:
        return self.lanes

## 16 `u8` lanes, 16 bytes (unsigned integer).
@derive(Copy)
pub struct u8x16:
    lanes: [u8; 16]

    pub const LANES: int = 16

extend u8x16:
    pub fn from_array(lanes: [u8; 16]) -> u8x16:
        return u8x16(lanes=lanes)

    pub fn to_array(self) -> [u8; 16]:
        return self.lanes

## 32 `u8` lanes, 32 bytes (unsigned integer).
@derive(Copy)
pub struct u8x32:
    lanes: [u8; 32]

    pub const LANES: int = 32

extend u8x32:
    pub fn from_array(lanes: [u8; 32]) -> u8x32:
        return u8x32(lanes=lanes)

    pub fn to_array(self) -> [u8; 32]:
        return self.lanes

## 64 `u8` lanes, 64 bytes (unsigned integer).
@derive(Copy)
pub struct u8x64:
    lanes: [u8; 64]

    pub const LANES: int = 64

extend u8x64:
    pub fn from_array(lanes: [u8; 64]) -> u8x64:
        return u8x64(lanes=lanes)

    pub fn to_array(self) -> [u8; 64]:
        return self.lanes

## 8 `i16` lanes, 16 bytes (signed integer).
@derive(Copy)
pub struct i16x8:
    lanes: [i16; 8]

    pub const LANES: int = 8

extend i16x8:
    pub fn from_array(lanes: [i16; 8]) -> i16x8:
        return i16x8(lanes=lanes)

    pub fn to_array(self) -> [i16; 8]:
        return self.lanes

## 16 `i16` lanes, 32 bytes (signed integer).
@derive(Copy)
pub struct i16x16:
    lanes: [i16; 16]

    pub const LANES: int = 16

extend i16x16:
    pub fn from_array(lanes: [i16; 16]) -> i16x16:
        return i16x16(lanes=lanes)

    pub fn to_array(self) -> [i16; 16]:
        return self.lanes

## 32 `i16` lanes, 64 bytes (signed integer).
@derive(Copy)
pub struct i16x32:
    lanes: [i16; 32]

    pub const LANES: int = 32

extend i16x32:
    pub fn from_array(lanes: [i16; 32]) -> i16x32:
        return i16x32(lanes=lanes)

    pub fn to_array(self) -> [i16; 32]:
        return self.lanes

## 8 `u16` lanes, 16 bytes (unsigned integer).
@derive(Copy)
pub struct u16x8:
    lanes: [u16; 8]

    pub const LANES: int = 8

extend u16x8:
    pub fn from_array(lanes: [u16; 8]) -> u16x8:
        return u16x8(lanes=lanes)

    pub fn to_array(self) -> [u16; 8]:
        return self.lanes

## 16 `u16` lanes, 32 bytes (unsigned integer).
@derive(Copy)
pub struct u16x16:
    lanes: [u16; 16]

    pub const LANES: int = 16

extend u16x16:
    pub fn from_array(lanes: [u16; 16]) -> u16x16:
        return u16x16(lanes=lanes)

    pub fn to_array(self) -> [u16; 16]:
        return self.lanes

## 32 `u16` lanes, 64 bytes (unsigned integer).
@derive(Copy)
pub struct u16x32:
    lanes: [u16; 32]

    pub const LANES: int = 32

extend u16x32:
    pub fn from_array(lanes: [u16; 32]) -> u16x32:
        return u16x32(lanes=lanes)

    pub fn to_array(self) -> [u16; 32]:
        return self.lanes

## 4 `i32` lanes, 16 bytes (signed integer).
@derive(Copy)
pub struct i32x4:
    lanes: [i32; 4]

    pub const LANES: int = 4

extend i32x4:
    pub fn from_array(lanes: [i32; 4]) -> i32x4:
        return i32x4(lanes=lanes)

    pub fn to_array(self) -> [i32; 4]:
        return self.lanes

## 8 `i32` lanes, 32 bytes (signed integer).
@derive(Copy)
pub struct i32x8:
    lanes: [i32; 8]

    pub const LANES: int = 8

extend i32x8:
    pub fn from_array(lanes: [i32; 8]) -> i32x8:
        return i32x8(lanes=lanes)

    pub fn to_array(self) -> [i32; 8]:
        return self.lanes

## 16 `i32` lanes, 64 bytes (signed integer).
@derive(Copy)
pub struct i32x16:
    lanes: [i32; 16]

    pub const LANES: int = 16

extend i32x16:
    pub fn from_array(lanes: [i32; 16]) -> i32x16:
        return i32x16(lanes=lanes)

    pub fn to_array(self) -> [i32; 16]:
        return self.lanes

## 4 `u32` lanes, 16 bytes (unsigned integer).
@derive(Copy)
pub struct u32x4:
    lanes: [u32; 4]

    pub const LANES: int = 4

extend u32x4:
    pub fn from_array(lanes: [u32; 4]) -> u32x4:
        return u32x4(lanes=lanes)

    pub fn to_array(self) -> [u32; 4]:
        return self.lanes

## 8 `u32` lanes, 32 bytes (unsigned integer).
@derive(Copy)
pub struct u32x8:
    lanes: [u32; 8]

    pub const LANES: int = 8

extend u32x8:
    pub fn from_array(lanes: [u32; 8]) -> u32x8:
        return u32x8(lanes=lanes)

    pub fn to_array(self) -> [u32; 8]:
        return self.lanes

## 16 `u32` lanes, 64 bytes (unsigned integer).
@derive(Copy)
pub struct u32x16:
    lanes: [u32; 16]

    pub const LANES: int = 16

extend u32x16:
    pub fn from_array(lanes: [u32; 16]) -> u32x16:
        return u32x16(lanes=lanes)

    pub fn to_array(self) -> [u32; 16]:
        return self.lanes

## 2 `i64` lanes, 16 bytes (signed integer).
@derive(Copy)
pub struct i64x2:
    lanes: [i64; 2]

    pub const LANES: int = 2

extend i64x2:
    pub fn from_array(lanes: [i64; 2]) -> i64x2:
        return i64x2(lanes=lanes)

    pub fn to_array(self) -> [i64; 2]:
        return self.lanes

## 4 `i64` lanes, 32 bytes (signed integer).
@derive(Copy)
pub struct i64x4:
    lanes: [i64; 4]

    pub const LANES: int = 4

extend i64x4:
    pub fn from_array(lanes: [i64; 4]) -> i64x4:
        return i64x4(lanes=lanes)

    pub fn to_array(self) -> [i64; 4]:
        return self.lanes

## 8 `i64` lanes, 64 bytes (signed integer).
@derive(Copy)
pub struct i64x8:
    lanes: [i64; 8]

    pub const LANES: int = 8

extend i64x8:
    pub fn from_array(lanes: [i64; 8]) -> i64x8:
        return i64x8(lanes=lanes)

    pub fn to_array(self) -> [i64; 8]:
        return self.lanes

## 2 `u64` lanes, 16 bytes (unsigned integer).
@derive(Copy)
pub struct u64x2:
    lanes: [u64; 2]

    pub const LANES: int = 2

extend u64x2:
    pub fn from_array(lanes: [u64; 2]) -> u64x2:
        return u64x2(lanes=lanes)

    pub fn to_array(self) -> [u64; 2]:
        return self.lanes

## 4 `u64` lanes, 32 bytes (unsigned integer).
@derive(Copy)
pub struct u64x4:
    lanes: [u64; 4]

    pub const LANES: int = 4

extend u64x4:
    pub fn from_array(lanes: [u64; 4]) -> u64x4:
        return u64x4(lanes=lanes)

    pub fn to_array(self) -> [u64; 4]:
        return self.lanes

## 8 `u64` lanes, 64 bytes (unsigned integer).
@derive(Copy)
pub struct u64x8:
    lanes: [u64; 8]

    pub const LANES: int = 8

extend u64x8:
    pub fn from_array(lanes: [u64; 8]) -> u64x8:
        return u64x8(lanes=lanes)

    pub fn to_array(self) -> [u64; 8]:
        return self.lanes

## A mask of 4 lanes of 32 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask32x4:
    lanes: [i32; 4]

    pub const LANES: int = 4

extend mask32x4:
    pub fn to_array(self) -> [bool; 4]:
        out: [bool; 4] = [false; 4]
        for i in 0..4:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 8 lanes of 32 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask32x8:
    lanes: [i32; 8]

    pub const LANES: int = 8

extend mask32x8:
    pub fn to_array(self) -> [bool; 8]:
        out: [bool; 8] = [false; 8]
        for i in 0..8:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 16 lanes of 32 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask32x16:
    lanes: [i32; 16]

    pub const LANES: int = 16

extend mask32x16:
    pub fn to_array(self) -> [bool; 16]:
        out: [bool; 16] = [false; 16]
        for i in 0..16:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 2 lanes of 64 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask64x2:
    lanes: [i64; 2]

    pub const LANES: int = 2

extend mask64x2:
    pub fn to_array(self) -> [bool; 2]:
        out: [bool; 2] = [false; 2]
        for i in 0..2:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 4 lanes of 64 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask64x4:
    lanes: [i64; 4]

    pub const LANES: int = 4

extend mask64x4:
    pub fn to_array(self) -> [bool; 4]:
        out: [bool; 4] = [false; 4]
        for i in 0..4:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 8 lanes of 64 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask64x8:
    lanes: [i64; 8]

    pub const LANES: int = 8

extend mask64x8:
    pub fn to_array(self) -> [bool; 8]:
        out: [bool; 8] = [false; 8]
        for i in 0..8:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 16 lanes of 8 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask8x16:
    lanes: [i8; 16]

    pub const LANES: int = 16

extend mask8x16:
    pub fn to_array(self) -> [bool; 16]:
        out: [bool; 16] = [false; 16]
        for i in 0..16:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 32 lanes of 8 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask8x32:
    lanes: [i8; 32]

    pub const LANES: int = 32

extend mask8x32:
    pub fn to_array(self) -> [bool; 32]:
        out: [bool; 32] = [false; 32]
        for i in 0..32:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 64 lanes of 8 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask8x64:
    lanes: [i8; 64]

    pub const LANES: int = 64

extend mask8x64:
    pub fn to_array(self) -> [bool; 64]:
        out: [bool; 64] = [false; 64]
        for i in 0..64:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 8 lanes of 16 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask16x8:
    lanes: [i16; 8]

    pub const LANES: int = 8

extend mask16x8:
    pub fn to_array(self) -> [bool; 8]:
        out: [bool; 8] = [false; 8]
        for i in 0..8:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 16 lanes of 16 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask16x16:
    lanes: [i16; 16]

    pub const LANES: int = 16

extend mask16x16:
    pub fn to_array(self) -> [bool; 16]:
        out: [bool; 16] = [false; 16]
        for i in 0..16:
            out[i] = self.lanes[i] != 0
        return out

## A mask of 32 lanes of 16 bits: a set lane is all ones, a clear one all zeros.
@derive(Copy)
pub struct mask16x32:
    lanes: [i16; 32]

    pub const LANES: int = 32

extend mask16x32:
    pub fn to_array(self) -> [bool; 32]:
        out: [bool; 32] = [false; 32]
        for i in 0..32:
            out[i] = self.lanes[i] != 0
        return out
