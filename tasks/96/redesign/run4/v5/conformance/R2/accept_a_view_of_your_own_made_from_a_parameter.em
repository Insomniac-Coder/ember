#$ test: run-pass
#$ rules: BRW-12
#$ stdout: 2
## design rules: R2
## R2: the nearest program that is allowed: a local view of your own; a name declared with `=` that
## holds a view may later view something else.

fn rest_len(line: StrView) -> int:
    rest = line.trim()
    return rest.len()

fn main():
    println(rest_len("  ab "))
