#$ test: parse-pass
#$ rules: STA-1
# A library module that gives a module variable a value with a statement: only the entry file is a
# script (`[GRM-2]`).

FRAME = 0

pub fn first() -> int:
    return 1
