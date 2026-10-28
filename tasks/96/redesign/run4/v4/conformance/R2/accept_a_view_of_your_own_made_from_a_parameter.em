#$ test: run-pass
#$ rules: R2
#$ stdout: 2
## R2: the nearest program that is allowed: a local view of your own; a name declared with `=` that
## holds a view may later view something else.

fn rest_len(line: StrView) -> int:
    rest = line.trim()
    return rest.len()

fn main():
    println(rest_len("  ab "))
