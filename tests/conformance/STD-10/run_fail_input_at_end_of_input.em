#$ test: run-fail
#$ rules: STD-10
#$ panics: input: end of input; `std.io.stdin().read_line()` returns it as a `Result` instead
# `[STD-10]` — a console failure is fatal: end of input panics, and the
# message names the `Result` form in `std.io`.

fn main():
    line = input()
    println(line)
