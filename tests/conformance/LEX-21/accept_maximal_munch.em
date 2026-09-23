#$ test: run-pass
#$ rules: LEX-21, TYP-28
#$ profiles: debug, release, shipping
#$ stdout: 3 3 3.5
# Tokenisation is maximal munch: `//=` before `//` before `/`.

fn main():
    x = 7
    x //= 2
    y = 7 // 2
    z = 7.0 / 2.0
    println(x, y, z)
