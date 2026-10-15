#$ test: run-pass
#$ rules: CT-7, CT-1
#$ stdout: ran
# `[CT-7]` — a `comptime:` block at item level or as a statement runs once while compiling, for
# its checks; nothing of it is left in the program.

fn table_size() -> int:
    return 64

comptime:
    assert(table_size() % 8 == 0, "the table is a whole number of words")

fn main():
    comptime:
        size = table_size()
        assert(size > 0 and size <= 1024)
    println("ran")
