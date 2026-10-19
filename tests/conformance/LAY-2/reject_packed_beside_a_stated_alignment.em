#$ test: compile-fail
#$ rules: LAY-2, PHIL-12
#$ profiles: debug
# `[LAY-2]` — `@packed` leaves no padding, while `@align(N)` makes a size a multiple of `N` and has
# every place holding its type honour `N`. The two meet on one type, or a packed type holds an
# `@align` type (directly or inside another value), and the specification does not say which
# wins: `E0901` (`[PHIL-12]`), never one chosen.

@align(16)
@derive(Copy)
struct Line:
    a: u8

@derive(Copy)
struct Wrapper:
    line: Line

@packed
@align(8)  #$ error[E0901]: `@align` beside `@packed` is not specified
struct Both:
    a: u8

@packed
struct Holds:
    a: u8
    line: Line  #$ error[E0901]: field `line` of a `@packed` type holds `Line`, whose alignment `@align` states

@packed
struct Deep:
    a: u8
    inner: (u8, Wrapper)  #$ error[E0901]: field `inner` of a `@packed` type holds

@packed
enum Payload:
    Empty
    Lined(Line)  #$ error[E0901]: field `_0` of a `@packed` type holds `Line`

fn main():
    println(size_of[Holds]())
