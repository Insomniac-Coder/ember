#$ test: run-pass
#$ rules: LEX-13
#$ stdout: 1
## `[LEX-13]` — `_` matches and discards, as often as it is written.

fn triple() -> (int, int, int):
    return (1, 2, 3)

fn main():
    (a, _, _) = triple()
    println(a)
