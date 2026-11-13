#$ test: run-pass
#$ rules: LEX-16, TYP-5
#$ stdout: 255 255 255 -32768 -128 7.0 3 1.0
## `[LEX-16]` - an unsuffixed integer literal takes the type its context expects: an initialiser, an
## argument, a field, a collection element, a return value, an operand; a float type too
## (`Vec3(1, 2, 3)`); with no context it is `int`. `-128` against `i8` is the negative constant.

struct Px:
    r: u8

struct Vec3:
    x: f32
    y: f32
    z: f32

fn mk() -> u8:
    return 255

fn take(x: i16) -> i16:
    return x

fn main():
    x: u8 = 200
    p = Px(255)
    xs: Array[i8] = [127, -128]
    f: f32 = 7
    v = Vec3(1, 2, 3)
    println(x + 55, p.r, mk(), take(-32768), xs[1], f, 1 + 2, v.x)
