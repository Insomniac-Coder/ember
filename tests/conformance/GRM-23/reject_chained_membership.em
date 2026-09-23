#$ test: parse-fail
#$ rules: GRM-23, GRM-25
# "Membership does not chain: `a in b in c` is `E0102`" ([GRM-23], 0.9.9).
# Comparisons chain as in Python ([GRM-25]); `is` and `in` do not.

fn main():
    xs: Array[i32] = Array[i32]()
    ys: Array[i32] = Array[i32]()
    b = 1 in xs in ys           #$ error[E0102]: `is` and `in` may not appear in a comparison chain
    println(1)
