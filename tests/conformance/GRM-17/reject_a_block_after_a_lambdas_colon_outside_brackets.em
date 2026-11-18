#$ test: compile-fail
#$ rules: GRM-17, GRM-28
## `[GRM-17]` — outside brackets a lambda's `:` body stands on the lambda's own line; a `NEWLINE` and
## `INDENT` after the `:` is `E0106`, whose message says the body is one simple statement and whose help
## is the local function to write (`fn add(x):` on the line above, then `h = add`, `[GRM-28]`).

fn main():
    total = 0
    h = fn(x: int):    #$ error[E0106]: a lambda's body is one simple statement; declare a local function
#$ help: fn add(x):
        total += x
    k = fn(x: int) -> int:    #$ error[E0106]: a lambda's body is one simple statement; declare a local function
        return x + 1
    println(total)
