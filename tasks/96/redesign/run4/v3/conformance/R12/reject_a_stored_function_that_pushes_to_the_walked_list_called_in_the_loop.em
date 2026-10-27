#$ test: compile-fail
#$ rules: R12, R8
#$ help: for k in 0..n: hs[0]()
## R8 (c), R12 (rules.md lines 553-556, 626-628): a call that may run a function value is a change of each
## real variable it changes, so it is refused inside a loop over that variable (E3020; the phase before's
## `attacks2.exe 11` showed the use after free). AN-58 part 6's message, in this program's lines.

fn main():
    xs = [1, 2]
    hs: Array[fn() changes xs] = []
    hs.push(fn() changes xs: xs.push(0))
    for x in xs:
        println(x)
        hs[0]()    #$ error[E3020]: this loop walks `xs` (line 12), and `hs[0]()` (line 14) may run the function made on line 11, which pushes to `xs`; call it after the loop, as many times as the loop would: write `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it
