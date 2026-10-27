#$ test: compile-fail
#$ rules: R12, R8
#$ help: move line 14 below the loop
## R8 (c), R12 (rules.md lines 711-714): driving an adapter that keeps a function which pushes to `seen`
## makes that change ("whatever drives the adapter ... makes its changes"), worked out from the lambda, so
## it is refused inside a loop over `seen` (E3020). AN-55 part 6's message, in this program's lines,
## naming the chain (R18). The nearest allowed program is accept_inspect_keeps_the_function_and_count_runs_it.

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = [0]
    for s in seen:
        println(s)
        _n = xs.iter().inspect(fn(v): seen.push(v)).count()    #$ error[E3020]: `xs.iter().inspect(...).count()` (line 14) adds to `seen` (`count` drives the adapter, which runs the function `inspect` keeps, which pushes to `seen`) while this loop walks `seen` (line 12); count after the loop: move line 14 below the loop
