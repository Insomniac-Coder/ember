#$ test: compile-fail
#$ rules: GRM-15, FN-2a
# `[GRM-15]` — `owns` binds as `-` does (`unary := value_word unary`, `conversion := unary {"as"
# type}`): `owns v as u8` is `(owns v) as u8`, a `u8` made after `v` is handed over, so `push` is
# not given its value with `owns`: `owns` is never an operand, and `as` is an operator of rows 4 to 17
# (`E0119`, `[GRM-47]`); in brackets, `owns (v as u8)`, it is.

fn main():
    out: Array[u8] = []
    v = 300
    out.push(owns (v as u8))
    out.push(owns v as u8)    #$ error[E0119]: `owns` here is an operand of `as`, which only reads `v`
    #$ help: write the product in brackets: `out.push(owns (v as u8))`
    println(out.len())
