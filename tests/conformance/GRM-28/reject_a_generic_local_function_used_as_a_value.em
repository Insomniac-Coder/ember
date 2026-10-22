#$ test: compile-fail
#$ rules: GRM-28, TYP-16, TYP-17
## `[GRM-28]` — a generic local function is a generic function: its name is no one function
## (`E2060`, as a module-level generic's is, `[TYP-16]`), only what its bounds provide is
## available in its body, checked once (`[TYP-17]`), and a call whose type argument misses a bound
## is refused at the call.

struct Plain:
    v: int

fn main():
    fn shown[T: Display](x: T) -> String:
        return f"{x}"
    fn bumped[T](x: T) -> T:
        return x + 1    #$ error[E2040]: `T`'s bounds do not provide a matching `Add` operator
    held = shown    #$ error[E2060]: `shown` is generic, so it is not one function
    typed: fn(int) -> String = shown    #$ error[E2060]: `shown` is generic, so it is not one function
    println(shown(shown))    #$ error[E2060]: `shown` is generic, so it is not one function
    println(shown(Plain(1)))    #$ error[E2040]: `Plain` does not implement `Display`, which `T` requires
    println(bumped(2))
