#$ test: compile-fail
#$ rules: GRM-18
# `[GRM-18]` — `;` never separates statements: `a = 1; b = 2` is `E0105`, whose help puts each
# statement on its own line.

fn main():
    a = 1; b = 2                    #$ error[E0105]: `;` is not a statement separator
    println(a, b)
