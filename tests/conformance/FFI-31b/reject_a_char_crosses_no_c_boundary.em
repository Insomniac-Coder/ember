#$ test: compile-fail
#$ rules: FFI-31b
# `[FFI-31b]` — `char` crosses no C boundary, directly or in a `@layout(c)` aggregate passed by value
# or behind a pointer (`E5054`): C may pass any `uint32_t`, a surrogate included, which no `char`
# holds. The message gives `u32` and `char.from_u32`.

@layout(c)
struct Glyph:
    code: char
    width: i32

pub extern "C" fn letter(code: char) -> i32:    #$ error[E5054]: `char` has no C equivalent, and C may pass any `uint32_t`; take `c: u32` and make the character with `char.from_u32(c)`
    return 1

pub extern "C" fn glyph_width(glyph: *Glyph) -> i32:    #$ error[E5054]
    return 2

# A struct that points at itself is looked into once.
@layout(c)
struct Link:
    next: *Link
    value: i32

pub extern "C" fn link_value(link: *Link) -> i32:
    return 3

pub extern "C" fn code_of(value: u32) -> u32:
    return value

fn main():
    println(code_of(65))
