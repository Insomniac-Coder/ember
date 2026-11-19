#$ test: run-pass
#$ rules: DIA-16
#$ stdout: 2
# `[DIA-16]` — the note is for a diagnostic that suggests a `shares` owner; a program with none has no diagnostic.

fn main():
    values = [1, 2]
    println(values.len())
