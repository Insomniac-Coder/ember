#$ test: compile-fail
#$ rules: CT-7
#$ error[E6004]: evaluating this `comptime` block panics: the table is a whole number of words
#$ error[E6004]: evaluating this `comptime` block panics: negative
# `[CT-7]` — a failed `assert` or any other panic in a `comptime:` block is `E6004`, which shows the
# panic message and the compile-time call chain.

fn table_size() -> int:
    return 60

fn checked(n: int) -> int:
    if n < 0:
        panic("negative")
    return n

comptime:
    assert(table_size() % 8 == 0, "the table is a whole number of words")

fn main():
    comptime:
        checked(-1)
    println("unreachable")
