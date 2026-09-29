#$ test: compile-fail
#$ rules: CTL-2, CTL-3b
# `[CTL-2]` — a loop holds its iterable's loan for the whole loop even when
# its pattern binds nothing from it (`_`): a counted loop over an adapter
# chain, or over Python's `enumerate`/`zip`, still reads each element, as the
# loop through `next` does.
fn main():
    xs: Array[int] = [1, 2, 3]
    for i, _ in enumerate(xs):
        xs.push(i) #$ error[E3020]: cannot mutate `xs` while it is borrowed by this loop
    for _ in xs.iter().take(2):
        xs.push(0) #$ error[E3020]: cannot mutate `xs` while it is borrowed by this loop
    ys: Array[int] = [7, 8]
    for _, y in xs.iter().zip(ys.iter()):
        xs.clear() #$ error[E3020]: cannot mutate `xs` while it is borrowed by this loop
        println(y)
