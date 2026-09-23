#$ test: compile-fail
#$ rules: LT-7, LT-1
#$ error[E3060]: `tmp` does not live long enough
# The callback's result borrows `tmp` for as long as the callee keeps it, which
# ends when the callee returns.

fn leak(f: fn(Span[int]) -> Span[int]) -> Span[int]:
    tmp = [41]
    return f(tmp)

fn main():
    println(leak(fn(s) => s)[0])
