#$ test: compile-fail
#$ rules: CT-2
#$ error[E6010]: evaluating `SOURCE` reads `../CT-1/accept_every_compile_time_position.em`, which is outside the package directory
# `[CT-2]` — `comptime.read_file` reads only inside the package directory (here this file's), so a
# dependency's build cannot read the builder's files.

const SOURCE: int = comptime.read_file("../CT-1/accept_every_compile_time_position.em").len()

fn main():
    println(SOURCE)
