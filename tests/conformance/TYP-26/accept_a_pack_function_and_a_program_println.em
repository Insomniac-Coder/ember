#$ test: run-pass
#$ rules: TYP-26, TYP-47
#$ stdout: 0 1 3
#$ mine 7
## `[TYP-26]` - a function whose last positional parameter takes a pack's values takes any number of
## arguments; `print` and `println` are such functions of the library, not known to the compiler by name, so a
## program may declare a `println` of its own in its module.

[Ts...]
fn count(values: Ts...) -> int:
    return values.len()

fn println(x: int):
    print("mine ", x, "\n", sep="", end="")

fn main():
    print(count(), count(1), count(1, "a", 2.5), end="\n")
    println(7)
