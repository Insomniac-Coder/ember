#$ test: compile-fail
#$ rules: GRM-15, FN-2a
# `[GRM-15]` — `owns` binds as `-` does (`unary := value_word unary`, `conversion := unary {"as"
# type}`): `owns v as u8` is `(owns v) as u8`, a `u8` made after `v` is handed over, so `push` is
# not given its value with `owns` (`E3028`); in brackets, `owns (v as u8)`, it is.

fn main():
    out: Array[u8] = []
    v = 300
    out.push(owns (v as u8))
    out.push(owns v as u8)    #$ error[E3028]: say so at the call: `out.push(owns (v as u8))`
    #$ error[E0900]: `owns` before a value here is not built by this compiler yet
    println(out.len())
