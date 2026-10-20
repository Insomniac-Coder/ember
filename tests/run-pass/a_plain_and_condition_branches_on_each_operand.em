#$ test: run-pass
#$ rules: EXP-3
#$ stdout: 20140609
#$ stdout: 3 2

## D-752 — a `while` or `if` condition made of `and`, `or` and `not` over plain operands branches on
## each operand in turn (`lower_branch`), as C's `&&` does: MSVC left the loop below rolled when the
## `and` was a `bool` temporary, 1.15x C. Still short-circuit: the second operand is not evaluated
## when the first decides.

fn main():
    count = 0
    for py in 0..600:
        for px in 0..600:
            x0 = (px as float) * 3.0 / 600.0 - 2.0
            y0 = (py as float) * 2.0 / 600.0 - 1.0
            x = 0.0
            y = 0.0
            i = 0
            while i < 200 and x * x + y * y <= 4.0:
                xt = x * x - y * y + x0
                y = 2.0 * x * y + y0
                x = xt
                i = i + 1
            count = count + i
    println(count)
    a = [1, 2, 3]
    n = 0
    k = 0
    while not (k >= 3 or a[k] > 5):
        n += 1
        k += 1
    m = 0
    if k == 3 and not (n < 3):
        m = 2
    println(n, m)
