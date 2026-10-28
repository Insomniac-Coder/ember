#$ test: run-pass
#$ rules: FFI-8, TYP-12
#$ stdout: Color.Red 1
#$ stdout: Color.Blue 4
#$ stdout: Color(9) 9 other
#$ stdout: true false
#$ stdout: Red 9
#$ stdout: some
# `[FFI-8]` — a C `enum` is `@repr(<underlying>) enum` with `@non_exhaustive`:
# any value of the underlying type may arrive from C, so a `match` on it has a
# `_` arm, `Debug` shows a value no variant names as `Color(9)` and `Display` as `9`, and an
# `Option[Color]` keeps a tag of its own (every bit pattern is a value)
# (ODR-137). The exported Ember function stands for C and returns raw `int`s.

@repr(i32)
@non_exhaustive
enum Color:
    Red = 1
    Green = 2
    Blue = 4

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(link_name="color_of_c")
    safe fn color_of(n: i32) -> Color
    @ffi(link_name="echo_c")
    safe fn echo(c: Color) -> i32

pub extern "C" fn color_of_c(n: i32) -> i32:
    return n

pub extern "C" fn echo_c(c: i32) -> i32:
    return c

fn name(c: Color) -> str:
    match c:
        Color.Red:
            return "red"
        Color.Green:
            return "green"
        Color.Blue:
            return "blue"
        _:
            return "other"

fn main():
    red = color_of(1)
    println(f"{red:?}", red as i32)
    blue = color_of(4)
    println(f"{blue:?}", echo(blue))
    odd = color_of(9)
    println(f"{odd:?}", odd as i32, name(odd))
    println(red == Color.Red, odd == Color.Blue)
    println(red, odd)
    maybe: Option[Color] = Some(color_of(0))
    match maybe:
        Some(_):
            println("some")
        None:
            println("none")
