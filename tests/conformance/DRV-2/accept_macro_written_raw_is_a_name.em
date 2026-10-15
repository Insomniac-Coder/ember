#$ test: run-pass
#$ rules: DRV-2, LEX-14
#$ stdout: 1
# `[DRV-2]` — the reserved word `macro` is a name when written `r#macro`.

fn main():
    r#macro = 1
    println(r#macro)
