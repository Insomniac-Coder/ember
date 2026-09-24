#$ test: run-pass
#$ rules: TYP-36, TYP-17
#$ stdout: <3> <[1, 2]> <a> Point(x=1) 5 1, 2
# `[TYP-36]` — a bound on `Display`, `Debug` or `Copy` is met by the types
# the table gives them: numbers, text and collections are `Display`, a
# struct is `Debug` field-wise, a number is `Copy`. A generic body formats a
# parameter whose bound says it can.

struct Point:
    x: int

fn show[T: Display](x: T) -> String:
    return f"<{x}>"

fn debug[T: Debug](x: T) -> String:
    return f"{x!r}"

fn copy_of[T: Copy](x: T) -> T:
    return x

println(show(3), show([1, 2]), show("a"), debug(Point(1)), copy_of(5), [1, 2].join(", "))
