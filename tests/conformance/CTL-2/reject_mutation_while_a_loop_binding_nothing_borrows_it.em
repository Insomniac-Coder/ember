#$ test: compile-fail
#$ rules: CTL-2, CTL-3b
# `[CTL-2]` — a loop holds its iterable's loan for the whole loop even when
# its pattern binds nothing from it (`_`): a counted loop over an adapter
# chain, or over Python's `enumerate`/`zip`, still reads each element, as the
# loop through `next` does.
fn main():
    xs: Array[int] = [1, 2, 3]
    for i, _ in enumerate(xs):
        xs.push(owns i) #$ error[E3020]: `xs.push(owns i)` (line 10) adds to `xs`, which this loop walks (line 9); collect what it adds in a list of your own: write `added: Array[i64] = []` before the loop, `added.push(owns i)` in place of line 10, and `xs.extend(owns added)` after the loop
    for _ in xs.iter().take(2):
        xs.push(owns 0) #$ error[E3020]: `xs.push(owns 0)` (line 12) adds to `xs`, which this loop walks (line 11); collect what it adds in a list of your own: write `added: Array[i64] = []` before the loop, `added.push(owns 0)` in place of line 12, and `xs.extend(owns added)` after the loop
    ys: Array[int] = [7, 8]
    for _, y in xs.iter().zip(owns ys.iter()):
        xs.clear() #$ error[E3020]: `xs.clear()` (line 15) changes `xs`, which this loop walks (line 14); change it after the loop: move line 15, `xs.clear()`, below the loop
        println(y)
