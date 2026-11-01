#$ test: compile-fail
#$ rules: GRM-40
#$ help: write the brackets after the name: `enum Shape[T: needs Copy]:`
#$ help: write the brackets after the name: `interface Scale[Rhs = Self]:`
#$ help: write the brackets after the name: `type Pair[T] = (int, int)`
#$ help: write the brackets after `extend`: `extend[T] Shape:`
## `[GRM-40]` — every line that begins with `[` and is not directly above a function, struct or class
## is `E0100`, one per line, with one message (verdict 155): above an enum, an interface or a `type`
## alias (whose brackets go after the name, as the help writes), above an `extend` block (after
## `extend`, as its help writes) or an attribute line, the first of two bracket lines, and a line
## in a body that begins with a list.

[T: needs Copy]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
enum Shape:
    Dot
    Line

[Rhs = Self]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
interface Scale:
    fn scale(by: int) -> int

[T]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
type Pair = (int, int)

[T]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
extend Shape:
    fn twice() -> int:
        return 2

[T]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
@inline
fn one() -> int:
    return 1

[U]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
[T]
fn first(xs: Array[T]) -> T:
    return xs[0]

fn main():
    x = 1
    [T: needs Copy]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
    [10, 20].len()    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
    [a, b] = [1, 2]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
    println(x, one(), first([3]))
