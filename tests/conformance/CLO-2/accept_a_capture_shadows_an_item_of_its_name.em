#$ test: run-pass
#$ rules: CLO-2, GRM-28
#$ profiles: debug, development, release
#$ stdout: 6 module
#$ stdout: local 2.5 6
## `[CLO-2]` — a local shadows a module's item of its name in its function, and in a closure there,
## which captures it: `twice` read in the lambda is the local `5`, not the function. Read as a
## value it was the function (`E2020`: `+` on a `fn(i64) -> i64`), though called it was the
## capture; a generic local function named as a module's function (`[GRM-28]`) is called from a
## closure the same way.

fn twice(x: int) -> int:
    return x * 2

fn shown(x: int) -> String:
    return "module"

fn main():
    twice = 5
    later = fn() -> int => twice + 1
    println(later(), shown(1))
    fn shown[T: Display](x: T) -> String:
        return f"{x}"
    call = fn() -> String => shown("local")
    println(call(), shown(2.5), later())
