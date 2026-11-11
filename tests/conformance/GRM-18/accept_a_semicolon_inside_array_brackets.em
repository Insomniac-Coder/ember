#$ test: run-pass
#$ rules: GRM-18
#$ stdout: 0 7
# `[GRM-18]` — `;` appears only inside `[T; N]` and `[v; N]`.

fn main():
    a: [int; 3] = [0; 3]
    b = [7; 4]
    println(a[2], b[3])
