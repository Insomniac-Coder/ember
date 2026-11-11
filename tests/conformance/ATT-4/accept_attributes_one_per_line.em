#$ test: run-pass
#$ rules: ATT-4, ATT-1
#$ stdout: 5
# `[ATT-4]` — one attribute per line: several on one item are written each on its own line.

@inline
@cold
fn five() -> int:
    return 5

fn main():
    println(five())
