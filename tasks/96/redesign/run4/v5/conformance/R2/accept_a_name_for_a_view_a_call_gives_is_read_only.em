#$ test: run-pass
#$ rules: BRW-12, BRW-16
#$ stdout: 2
## design rules: R2, R8
## R2: a name bound to a call whose result is a view is a read-only view of it, and the temporary it
## views lives until the view's last use (row 5: `word = line.to_lower().trim()`).

fn length_of(line: StrView) -> int:
    word = line.to_lower().trim()
    return word.len()

fn main():
    println(length_of("  Ab "))
