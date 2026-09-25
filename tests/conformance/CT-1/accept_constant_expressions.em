#$ test: run-pass
#$ rules: CT-1, CT-4, STR-7, TYP-8, TYP-10, TYP-29
#$ profiles: debug, release, shipping
#$ stdout: 0 640 480 2 10000 10.0
#$ (-4, 1) 131 -16 0.33333334 0.3333333333333333 0.3333 -9223372036854775807
#$ true true 3 -4.0 Size(w=640, h=480)
# V.7, `[CT-1]` — a `const` may be any constant expression this phase
# evaluates: literals, other constants, arithmetic, comparisons, and the
# construction of tuples, arrays, structs and enums. It is worked out while
# compiling, each operation as the target does it (`[CT-4]`): `//` and `%`
# are floor division and modulo, `>>` on a signed type is arithmetic, and an
# `f32` or `f16` quotient is rounded to its type. Written without `: T`, a
# constant has its value's type, and an untyped numeric literal stays
# untyped (ODR-037). A type's `const` is named through the type, or through
# `Self` inside it (`[STR-7]`).

@derive(Copy)
struct Size:
    w: i32
    h: i32

    const UNIT: Size = Size(1, 1)
    const AREA_LIMIT: i32 = 100 * 100
    const SIDES = 4

    fn doubled(self) -> Size:
        return Size(self.w * Self.UNIT.w * 2, self.h * 2)

enum Mode:
    Fast
    Slow

const ORIGIN = Size(0, 0)
const BIG: Size = Size(Size.UNIT.w * 640, 480)
const PAIR: (i64, i64) = (-7 // 2, -7 % 2)
const BITS: u8 = (1 << 7) | 3
const SHIFTED: i8 = -128 >> 3
const THIRD: f32 = 1.0 / 3.0
const WIDE: f64 = 1.0 / 3.0
const THIRD_F16: f16 = 1.0 / 3.0
const SMALL = i64.MIN + 1
const CHOSEN: Mode = Mode.Slow
const OK: bool = BITS > 128 and not (THIRD == 0.0)
const TABLE: [i32; 3] = [1, 2, 3]
const FLOORED: f64 = -7.5 // 2.0

fn main():
    println(ORIGIN.w, BIG.w, BIG.h, Size.UNIT.doubled().w, Size.AREA_LIMIT, Size.SIDES * 2.5)
    println(PAIR, BITS, SHIFTED, THIRD, WIDE, THIRD_F16, SMALL)
    println(CHOSEN == Mode.Slow, OK, TABLE[2], FLOORED, BIG)
