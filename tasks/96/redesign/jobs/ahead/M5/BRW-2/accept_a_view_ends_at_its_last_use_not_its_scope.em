# Clause BRW-2.1-2: "A view is in use from where it is made to the last use of anything made from it
# (...). Scope does not matter: `n = v.len(); v.push(owns n)` is legal."
#$ test: run-pass
#$ rules: BRW-2, BCK-2, BCK-6, LT-5
#$ stdout: [1, 2, 3, 3]
#$ stdout: 2
#$ stdout: [1, 2, 9]
#$ stdout: 1
#$ stdout: 9
# The spec's line first; then views still in scope, and what was made from them, whose last use comes
# before each change.

fn main():
    v = [1, 2, 3]
    n = v.len()
    v.push(owns n)
    println(v)
    xs = [1, 2]
    s = xs.as_span()
    t = s[1..]
    println(t[0])
    xs.push(owns 9)
    println(xs)
    x = 1
    r = ref x
    println(r)
    x = 9
    println(x)
