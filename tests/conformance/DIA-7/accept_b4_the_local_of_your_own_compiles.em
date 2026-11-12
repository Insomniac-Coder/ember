#$ test: run-pass
#$ rules: DIA-7, DIA-25
#$ stdout: 2
# `[DIA-7]`, shape B4 — the line the help gives for a rebound parameter compiles: a local of your own
# first (`new_count = count`), changed and used from there on.

fn bump(count: int) -> int:
    new_count = count
    new_count = new_count + 1
    return new_count

fn main():
    println(bump(1))
