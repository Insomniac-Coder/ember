#$ test: run-pass
#$ rules: GRM-12
#$ stdout: bound
## `[GRM-12]` — a pattern name that is not a variant of the matched enum binds
## a new name, which matches anything; one spelled like a variant of another
## enum in scope is `W1002`, since a reader takes it for that variant (D-578).

enum Color:
    Red
    Green

enum Light:
    Off
    On(int)

fn describe(l: Light) -> str:
    return "bound"

fn show(l: Light):
    match l:
        Red:    #$ warning[W1002]: `Red` binds a new name, which matches any value
            println(describe(Red))

fn main():
    show(Light.Off)
