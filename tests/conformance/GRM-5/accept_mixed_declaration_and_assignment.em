#$ test: run-pass
#$ rules: GRM-5, GRM-4
#$ stdout: 2 3
## `[GRM-5]` (0.9.20) — each target follows `[GRM-4]` on its own, so a name in scope is assigned and one
## that is not is declared in the same line (0.9.10 refused the mixture).

fn main():
    existing = 1
    existing, fresh = (2, 3)
    println(existing, fresh)
