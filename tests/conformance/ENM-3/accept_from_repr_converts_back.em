#$ test: run-pass
#$ rules: ENM-3
#$ profiles: debug, development, release
#$ stdout: 1 Some(Level.High) None
#$ stdout: Some(Level.Low) Some(Shade.Dark) None
#$ stdout: Some(Code(9))
# `[ENM-3]` — a unit-only enum converts to its representation with `as`, and back with
# `from_repr(x) -> Option[E]`: `Some` of the variant whose discriminant `x` is (negative ones too),
# `None` when none is. A C enum (`@non_exhaustive`) holds every value of its type: always `Some`.

@repr(i16)
enum Level:
    Low = -3
    Mid = 0
    High = 7

enum Shade:
    Light
    Dark

@repr(i32)
@non_exhaustive
enum Code:
    Ok = 0
    Busy = 2

fn main():
    println(Level.Mid as i16 + 1, Level.from_repr(7), Level.from_repr(2))
    println(Level.from_repr(-3), Shade.from_repr(1), Shade.from_repr(5))
    println(Code.from_repr(9))
