#$ test: compile-fail
#$ rules: DIA-7, DIA-25, FN-2
#$ error[E3023]: `count` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_count = count + 1` in place of line 10, and `new_count` for `count` in line 11
#$ help: or write `ref mut count: int` for the parameter, and `ref mut` before the argument at each call, if the caller is to see the new value
# `[DIA-7]`, shape B4 (ODR-235) — a parameter given a new value (Python's rebinding of its name) has
# the line to write: a local of your own first (`count = n`), the parameter as the program wrote it
# where the caller is to see the change.

fn bump(count: int) -> int:
    count = count + 1
    return count

fn main():
    println(bump(1))
