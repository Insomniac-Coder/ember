#$ test: compile-fail
#$ rules: GRM-17, LEX-6a
## "A block-bodied lambda inside brackets whose body is not a single
## `small_stmt` is `E0106`, with `help: bind it on a preceding line:
## `h = fn(e): …` then pass `h`` and `note: indentation is not significant
## inside brackets ([LEX-6])`."

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    println(apply(fn(x: i32): y = x * 2      #$ error[E0106]: a multi-statement closure cannot be written inside brackets
        return y, 4))
