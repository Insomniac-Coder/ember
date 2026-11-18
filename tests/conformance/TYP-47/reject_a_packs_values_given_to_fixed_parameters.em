#$ test: compile-fail
#$ rules: TYP-47
#$ error[E2021]: `pair(values...)` gives a pack's values, any number of them, and `pair` takes two: `fn pair(a: int, b: int) -> int`
## `[TYP-47]` - a body is checked once, before its instances, so a pack's values, whose number is not known
## there, fit no parameters of a fixed number: given to them, the call is `E2021` (shape N6), whose message
## gives the callee's first line.

fn pair(a: int, b: int) -> int:
    return a + b

[Ts...]
fn forward(values: Ts...) -> int:
    return pair(values...)

fn main():
    println(forward(1, 2))
