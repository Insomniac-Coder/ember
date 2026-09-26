#$ test: run-pass
#$ rules: TYP-9, LEX-17, LEX-17a
#$ stdout: true true
#$ stdout: true true
#$ stdout: true true
# `[LEX-17]`, `[TYP-9]` (D-319) — a literal that takes a float type is rounded
# to it once, from what is written. Each literal here is just past the
# midpoint between two values of its type, by less than an `f64` can tell:
# rounded to an `f64` first it lands on the midpoint, and the second rounding
# (ties to even) went down. Parsing the same text rounds once, and agrees.

fn main():
    x: f32 = 1.00000005960464477539062500000001 #$ warning[W2015]
    y = 1.00000005960464477539062500000001f32
    println(x == 1.0000001, y == "1.00000005960464477539062500000001".parse[f32]().unwrap())
    h: f16 = 1.00048828125000000000001 #$ warning[W2015]
    println(h > 1.0, h - 1.0 > 0.0009)
    big: f32 = 1152921573326323713
    println(big == 1152921642045800448, big == "1152921573326323713".parse[f32]().unwrap())
