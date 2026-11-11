#$ test: compile-fail
#$ rules: EFF-18, EFF-20, EFF-5
#$ error[E4041]: `@noio` function `ask` reaches I/O
#$ error[E4003]: `@noblock` function `ask_again` reaches an operation that may wait
# `[EFF-18]` — effects are independent: a blocking read has `Io + Block`, so `@noio` refuses it
# for its `Io` and `@noblock` refuses it for its `Block`.

@noio
fn ask() -> String:
    return input()

@noblock
fn ask_again() -> String:
    return input()

fn main():
    println(ask(), ask_again())
