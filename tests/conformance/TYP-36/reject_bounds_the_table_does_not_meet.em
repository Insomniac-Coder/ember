#$ test: compile-fail
#$ rules: TYP-36, STD-15
#$ note: `join` needs `Point` to implement `Display`
# `[TYP-36]` — a `—` in the table means the implementation does not exist, and
# a bound requiring it is `E2040`: a struct is `Display` only when it says so,
# and a `String` is not `Copy`.

struct Point:
    x: int

fn show[T: Display](x: T) -> String:
    return f"<{x}>"

fn copy_of[T: Copy](x: T) -> T:
    return x

println(show(Point(1)))   #$ error[E2040]: `Point` does not implement `Display`, which `T` requires
println(copy_of(String.from("s")))   #$ error[E2040]: `String` does not implement `Copy`, which `T` requires
println([Point(1)].join(", "))   #$ error[E1010]: `Array[Point]` has no method named `join`
