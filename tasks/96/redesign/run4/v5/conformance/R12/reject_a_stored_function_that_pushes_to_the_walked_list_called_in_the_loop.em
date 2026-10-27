#$ test: compile-fail
#$ rules: R12, R8
#$ help: write `n = xs.len()` above line 14
## R8 (c), R12 (rules.md lines 717-720, 752-756): "a call that may run [the function value is] a change of
## each [real variable] it changes", so `hs[0]()` is refused inside a loop over `xs` (E3020; the phase
## before's `attacks2.exe 11` showed the use after free). AN-58 part 6's message, in this program's lines,
## naming the chain and every line it replaces (R18). The nearest allowed program is
## accept_the_stored_function_called_after_the_loop_as_many_times.

fn main():
    xs = [1, 2]
    hs: Array[fn()] = []
    hs.push(fn(): xs.push(0))
    for x in xs:
        println(x)
        hs[0]()    #$ error[E3020]: `hs[0]()` (line 16) adds to `xs` (it runs the function stored on line 13, which pushes to `xs`) while this loop walks `xs` (line 14); call it after the loop, as many times as the loop would: write `n = xs.len()` above line 14, remove line 16, and below the loop write `for k in 0..n:` with `hs[0]()` under it
