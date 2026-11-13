#$ test: compile-fail
#$ rules: GRM-17, LEX-6a
## `[GRM-17]` — a lambda with a `:` body inside brackets holds exactly one simple statement; more is
## `E0106`, whose help is to declare a local function on the preceding line (`[GRM-28]`), with the note
## that indentation is not significant inside brackets (`[LEX-6]`).

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    println(apply(fn(x: i32): y = x * 2      #$ error[E0106]: a multi-statement closure cannot be written inside brackets
#$ help: declare a local function on the preceding line
        return y, 4))
